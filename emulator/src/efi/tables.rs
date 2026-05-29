use crate::bus::SystemBus;
use crate::constants::*;
use super::encode::{movz_x, movk_x, write64, write32};
use super::protocols::{loaded_image_protocol_addr, LOADED_IMAGE_GUID_LO};

/// Encode a `RET X30` instruction.
fn encode_ret() -> u32 { INSTR_RET }

/// Write a sequence of 32-bit ARM64 instructions to memory.
fn write_trampoline(bus: &mut SystemBus, addr: u64, insts: &[u32]) {
    for (i, &inst) in insts.iter().enumerate() {
        write32(bus, addr + (i as u64 * INSTRUCTION_SIZE), inst);
    }
}

/// Encode the bump-allocator trampoline (AllocatePool).
///
/// Allocates memory from a linear pool by bumping a head pointer.
///   X0 ← size (in bytes)
///   X2 ← **Buffer (where to write the pointer)
///   Returns EFI_SUCCESS (0).
fn bump_allocator_trampoline(head_ptr: u64) -> [u32; 8] {
    [
        movz_x(4, (head_ptr & 0xFFFF) as u16),
        movk_x(4, 1, ((head_ptr >> 16) & 0xFFFF) as u16),
        0xF9400085,              // LDR  X5, [X4]      // read current pool head
        0x8B0100A0,              // ADD  X0, X5, X1    // X0 = head + size (X1=Size)
        0xF9000080,              // STR  X0, [X4]      // update pool head
        0xF9000045,              // STR  X5, [X2]      // *Buffer = old head (X2=**Buffer)
        movz_x(0, 0),            // MOVZ X0, #0        // EFI_SUCCESS
        encode_ret(),
    ]
}

/// Build a MOVZ/MOVK sequence to materialize a 64-bit constant in register `rd`.
fn encode_mov64(insts: &mut Vec<u32>, rd: u8, val: u64) {
    insts.push(movz_x(rd, (val & 0xFFFF) as u16));
    if val >> 16 != 0 {
        insts.push(movk_x(rd, 1, ((val >> 16) & 0xFFFF) as u16));
    }
    if val >> 32 != 0 {
        insts.push(movk_x(rd, 2, ((val >> 32) & 0xFFFF) as u16));
    }
    if val >> 48 != 0 {
        insts.push(movk_x(rd, 3, ((val >> 48) & 0xFFFF) as u16));
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Large trampolines — complex EFI services (> 32 bytes of code)
// ═══════════════════════════════════════════════════════════════════

/// Build the GetMemoryMap trampoline.
///
/// Reports a single EFI_MEMORY_DESCRIPTOR covering the entire RAM region.
/// If the caller's buffer is too small, returns EFI_BUFFER_TOO_SMALL.
fn build_get_memory_map_trampoline() -> Vec<u32> {
    let mut v = Vec::new();

    // X6 = *MemoryMapSize (current buffer size)
    v.push(0xf9400006); // LDR X6, [X0]

    // CMP X6, #48  (SUBS XZR, X6, #48) — is the buffer big enough?
    v.push(0xF100C0DF);

    // B.HS label_fill — if X6 >= 48, skip to "fill" logic
    v.push(0x54000002); // placeholder, patched below

    // ── label_too_small: buffer isn't big enough ──
    encode_mov64(&mut v, 5, EFI_MEMORY_DESC_SIZE);   // *MemoryMapSize = 48
    v.push(0xf9000005); // STR X5, [X0]
    v.push(0xf9000065); // *DescriptorSize = 48
    encode_mov64(&mut v, 5, EFI_MEMORY_DESC_VERSION);
    v.push(0xb9000085); // *DescriptorVersion = 1

    encode_mov64(&mut v, 0, EFI_BUFFER_TOO_SMALL);
    v.push(encode_ret());

    let too_small_len = v.len() as i32;

    // ── label_fill: write one EFI_MEMORY_DESCRIPTOR at [X1] ──
    // Type = EfiConventionalMemory (7)
    encode_mov64(&mut v, 5, EFI_CONVENTIONAL_MEMORY_TYPE);
    v.push(0xb9000025); // STR W5, [X1]        // Type
    v.push(0xb900043f); // STR WZR, [X1, #4]  // Pad = 0
    encode_mov64(&mut v, 5, RAM_BASE);
    v.push(0xf9000425); // STR X5, [X1, #8]   // PhysicalStart
    v.push(0xf900083f); // STR XZR, [X1, #16] // VirtualStart = 0
    encode_mov64(&mut v, 5, 0x40000u64);      // NumberOfPages = 1 GiB / 4 KiB
    v.push(0xf9000c25); // STR X5, [X1, #24]  // NumberOfPages
    encode_mov64(&mut v, 5, EFI_MEMORY_WB);
    v.push(0xf9001025); // STR X5, [X1, #32]  // Attribute
    v.push(0xf900143f); // STR XZR, [X1, #40] // Pad2 = 0

    // Set outputs
    encode_mov64(&mut v, 5, EFI_MEMORY_DESC_SIZE);
    v.push(0xf9000005); // *MemoryMapSize = 48
    encode_mov64(&mut v, 5, EFI_MEMORY_MAP_KEY);
    v.push(0xf9000045); // *MapKey = 17
    encode_mov64(&mut v, 5, EFI_MEMORY_DESC_SIZE);
    v.push(0xf9000065); // *DescriptorSize = 48
    encode_mov64(&mut v, 5, EFI_MEMORY_DESC_VERSION);
    v.push(0xb9000085); // *DescriptorVersion = 1
    v.push(movz_x(0, 0)); // EFI_SUCCESS
    v.push(encode_ret());

    // Patch the B.HS branch target
    let branch_offset = (too_small_len - 2) as u32;
    let bcond_hs = 0x54000002u32 | ((branch_offset & 0x7FFFF) << 5);
    v[2] = bcond_hs;

    v
}

/// Build the HandleProtocol / OpenProtocol trampoline.
///
/// Checks if the requested GUID matches EFI_LOADED_IMAGE_PROTOCOL.
/// If yes: *Interface = LIP address, returns EFI_SUCCESS.
/// If no:  *Interface = NULL, returns EFI_NOT_FOUND.
fn build_handle_protocol_trampoline(lip_addr: u64) -> Vec<u32> {
    let mut v = Vec::new();

    // LDR X4, [X1] — load first 8 bytes of GUID
    v.push(0xf9400024);
    // Build expected GUID low bits into X3
    encode_mov64(&mut v, 3, LOADED_IMAGE_GUID_LO);
    // SUBS X4, X4, X3 — compare
    v.push(0xEB030084);
    // CBNZ X4, label_not_found — placeholder, patched below
    let cbnz_idx = v.len();
    v.push(0xB5000004);

    // ── GUID matches: return LIP ──
    encode_mov64(&mut v, 3, lip_addr);
    v.push(0xf9000043); // STR X3, [X2]
    v.push(movz_x(0, 0)); // EFI_SUCCESS
    v.push(encode_ret());

    let not_found_idx = v.len();

    // ── GUID doesn't match ──
    v.push(0xF900005F); // STR XZR, [X2]  // *Interface = NULL
    encode_mov64(&mut v, 0, EFI_NOT_FOUND);
    v.push(encode_ret());

    // Patch CBNZ X4: offset to not_found label
    let offset = (not_found_idx as i32 - cbnz_idx as i32) as u32;
    v[cbnz_idx] = 0xB5000004u32 | ((offset & 0x7FFFF) << 5);

    v
}

/// LocateProtocol trampoline: return EFI_NOT_FOUND, *Interface = NULL.
fn build_locate_protocol_trampoline() -> Vec<u32> {
    let mut v = Vec::new();
    v.push(0xB4000042); // CBZ X2, skip_store — guard null pointer
    v.push(0xF900005F); // STR XZR, [X2] — *Interface = NULL
    encode_mov64(&mut v, 0, EFI_NOT_FOUND);
    v.push(encode_ret());
    v
}

// ═══════════════════════════════════════════════════════════════════
//  Main EFI table setup
// ═══════════════════════════════════════════════════════════════════

/// Build and install all UEFI firmware structures into the system bus.
///
/// Returns `(image_handle, system_table_address)` — these are loaded into
/// X0/X1 before jumping to the EFI entry point.
pub fn setup_efi_tables(
    bus: &mut SystemBus,
    image_base: u64,
    image_size: u64,
    dtb_addr: u64,
) -> (u64, u64) {
    let handle = EFI_HANDLE_ADDR;

    // Store image handle
    write64(bus, handle, 0x1_0000);
    write64(bus, EFI_SYSTEM_TABLE_PTR_ADDR, EFI_SYSTEM_TABLE_ADDR);

    let st = EFI_SYSTEM_TABLE_ADDR;

    // ── EFI System Table header ──
    write64(bus, st + 0x00, EFI_ST_SIGNATURE);
    write32(bus, st + 0x08, EFI_ST_REVISION);
    write32(bus, st + 0x0C, EFI_ST_HEADER_SIZE);
    write32(bus, st + 0x10, 0); // CRC32 (not calculated)
    write32(bus, st + 0x14, 0); // Reserved

    // ── Console handles ──
    let con_out_struct = CONSOLE_OUT_STRUCT_ADDR;
    let con_out_handle = CONSOLE_OUT_HANDLE_ADDR;
    write64(bus, st + 0x18, 0);                // FirmwareVendor
    write32(bus, st + 0x20, 0);                // FirmwareRevision
    write32(bus, st + 0x24, 0);                // Padding
    write64(bus, st + 0x28, 0);                // ConsoleInHandle
    write64(bus, st + 0x30, 0);                // ConIn
    write64(bus, st + 0x38, con_out_handle);   // ConsoleOutHandle
    write64(bus, st + 0x40, con_out_struct);   // ConOut
    write64(bus, st + 0x48, con_out_handle);   // StandardErrorHandle
    write64(bus, st + 0x50, con_out_struct);   // StdErr
    write64(bus, st + 0x58, EFI_RUNTIME_SERVICES_ADDR);
    write64(bus, st + 0x60, EFI_BOOT_SERVICES_ADDR);

    // ── FDT configuration table ──
    let config_table = EFI_CONFIG_TABLE_ADDR;
    // FDT GUID: b1b621d5-f19c-41a5-830b-d9152c69aae0
    write64(bus, config_table + 0,  0x41a5_f19c_b1b6_21d5);
    write64(bus, config_table + 8,  0xe0aa_692c_15d9_0b83);
    write64(bus, config_table + 16, dtb_addr);

    write64(bus, st + 0x68, EFI_MEMORY_DESC_COUNT);   // NumberOfTableEntries
    write64(bus, st + 0x70, config_table);             // ConfigurationTable

    // ── Runtime Services (14 entries, all "return EFI_SUCCESS") ──
    let rt_offsets: [u64; 14] = [
        0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48,
        0x50, 0x58, 0x60, 0x68, 0x70, 0x78, 0x80,
    ];
    for (i, &off) in rt_offsets.iter().enumerate() {
        let trampoline_addr = EFI_TRAMPOLINES_ADDR + (i as u64) * TRAMPOLINE_SLOT_SIZE;
        let ptr = super::encode::write_success_trampoline(bus, trampoline_addr, EFI_SUCCESS);
        write64(bus, EFI_RUNTIME_SERVICES_ADDR + off, ptr);
    }

    // ── Boot Services (44 entries) ──
    // Indexed at 256+ to avoid clashing with runtime trampoline slots.
    let boot_offsets: &[u64] = &[
        0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60,
        0x68, 0x70, 0x78, 0x80, 0x88, 0x90, 0x98, 0xA0, 0xA8, 0xB0,
        0xB8, 0xC0, 0xC8, 0xD0, 0xD8, 0xE0, 0xE8, 0xF0, 0xF8, 0x100,
        0x108, 0x110, 0x118, 0x120, 0x128, 0x130, 0x138, 0x140, 0x148, 0x150,
        0x158, 0x160, 0x168, 0x170,
    ];
    for (i, &off) in boot_offsets.iter().enumerate() {
        let trampoline_addr = EFI_TRAMPOLINES_ADDR + (256 + i as u64) * TRAMPOLINE_SLOT_SIZE;
        let ptr = super::encode::write_success_trampoline(bus, trampoline_addr, EFI_SUCCESS);
        write64(bus, EFI_BOOT_SERVICES_ADDR + off, ptr);
    }

    // ── Specialised large trampolines ──
    // Each occupies one LARGE_CODE_BLOCK_SIZE block (512 bytes).

    // Block 0: AllocatePool — bump allocator
    let allocpool_tp = EFI_LARGE_CODE_ADDR + 0 * LARGE_CODE_BLOCK_SIZE;
    let bump = bump_allocator_trampoline(EFI_POOL_HEAD_PTR);
    write_trampoline(bus, allocpool_tp, &bump);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_ALLOCATE_POOL_OFFSET, allocpool_tp);
    write64(bus, EFI_POOL_HEAD_PTR, EFI_POOL_BASE); // prime the pool head

    // Block 1: GetMemoryMap
    let memmap_tp = EFI_LARGE_CODE_ADDR + 1 * LARGE_CODE_BLOCK_SIZE;
    let memmap = build_get_memory_map_trampoline();
    assert!(memmap.len() * 4 <= LARGE_CODE_BLOCK_SIZE as usize,
            "GetMemoryMap trampoline too large: {} instructions", memmap.len());
    write_trampoline(bus, memmap_tp, &memmap);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_GET_MEMORY_MAP_OFFSET, memmap_tp);

    // Block 2: HandleProtocol
    let lip_addr = loaded_image_protocol_addr();
    let hp_code = build_handle_protocol_trampoline(lip_addr);
    let hp_tp = EFI_LARGE_CODE_ADDR + 2 * LARGE_CODE_BLOCK_SIZE;
    assert!(hp_code.len() * 4 <= LARGE_CODE_BLOCK_SIZE as usize,
            "HandleProtocol trampoline too large: {} instructions", hp_code.len());
    write_trampoline(bus, hp_tp, &hp_code);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_HANDLE_PROTOCOL_OFFSET, hp_tp);

    // Block 3: OpenProtocol — reuses HandleProtocol code
    let op_tp = EFI_LARGE_CODE_ADDR + 3 * LARGE_CODE_BLOCK_SIZE;
    write_trampoline(bus, op_tp, &hp_code);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_OPEN_PROTOCOL_OFFSET, op_tp);

    // Block 4: LocateProtocol — returns EFI_NOT_FOUND
    let lp_code = build_locate_protocol_trampoline();
    let lp_tp = EFI_LARGE_CODE_ADDR + 4 * LARGE_CODE_BLOCK_SIZE;
    write_trampoline(bus, lp_tp, &lp_code);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_LOCATE_PROTOCOL_OFFSET, lp_tp);

    // Block 5: ConOut.Reset — return EFI_SUCCESS
    let reset_tp = EFI_LARGE_CODE_ADDR + 5 * LARGE_CODE_BLOCK_SIZE;
    super::encode::write_success_trampoline(bus, reset_tp, EFI_SUCCESS);
    write64(bus, con_out_struct + 0x00, reset_tp);

    // Block 6: ConOut.OutputString — writes UCS-2 to UART byte-by-byte
    let output_tp = EFI_LARGE_CODE_ADDR + 6 * LARGE_CODE_BLOCK_SIZE;
    let output_insts = [
        0x79400022, // LDRH W2, [X1]          // load next UTF-16 character
        0x350000a2, // CBZ W2, #20            // null-terminator? → done
        0xD2A12003, // MOVZ X3, #0x0900, LSL #16
        0x380000a2, // STRB W2, [X3]          // write low byte to UART
        0x91000821, // ADD X1, X1, #2         // advance string pointer
        0x17FFFFFB, // B -20                   // loop
        0xD2800000, // MOVZ X0, #0            // EFI_SUCCESS
        INSTR_RET,
    ];
    write_trampoline(bus, output_tp, &output_insts);
    write64(bus, con_out_struct + 0x08, output_tp);

    // Block 7: CopyMem — byte-by-byte memory copy (X0=Dest, X1=Src, X2=Length)
    let copymem_tp = EFI_LARGE_CODE_ADDR + 7 * LARGE_CODE_BLOCK_SIZE;
    let copymem_insts = [
        0xB40000A2, // CBZ X2, #20             // length == 0? → done
        0x38401823, // LDRB W3, [X1], #1      // load byte from src, post-increment
        0x38001803, // STRB W3, [X0], #1      // store byte to dst, post-increment
        0xF1000442, // SUBS X2, X2, #1        // decrement count
        0x17FFFFFC, // B -16                   // loop
        0xD2800000, // MOVZ X0, #0            // EFI_SUCCESS
        INSTR_RET,
    ];
    write_trampoline(bus, copymem_tp, &copymem_insts);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_COPY_MEM_OFFSET, copymem_tp);

    // Block 8: SetMem — byte-by-byte memory fill (X0=Buffer, X1=Size, X2=Value)
    let setmem_tp = EFI_LARGE_CODE_ADDR + 8 * LARGE_CODE_BLOCK_SIZE;
    let setmem_insts = [
        0xB4000081, // CBZ X1, #16             // size == 0? → done
        0x38001802, // STRB W2, [X0], #1      // store byte, post-increment
        0xF1000421, // SUBS X1, X1, #1        // decrement count
        0x17FFFFFD, // B -12                   // loop
        0xD2800000, // MOVZ X0, #0            // EFI_SUCCESS
        INSTR_RET,
    ];
    write_trampoline(bus, setmem_tp, &setmem_insts);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_SET_MEM_OFFSET, setmem_tp);

    // Block 9: AllocatePages — return EFI_SUCCESS
    super::encode::write_success_trampoline(bus,
        EFI_LARGE_CODE_ADDR + 9 * LARGE_CODE_BLOCK_SIZE, EFI_SUCCESS);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_ALLOCATE_PAGES_OFFSET,
        EFI_LARGE_CODE_ADDR + 9 * LARGE_CODE_BLOCK_SIZE);

    // Block 10: FreePages — return EFI_SUCCESS
    super::encode::write_success_trampoline(bus,
        EFI_LARGE_CODE_ADDR + 10 * LARGE_CODE_BLOCK_SIZE, EFI_SUCCESS);
    write64(bus, EFI_BOOT_SERVICES_ADDR + BS_FREE_PAGES_OFFSET,
        EFI_LARGE_CODE_ADDR + 10 * LARGE_CODE_BLOCK_SIZE);

    // The EFI stub dereferences a NULL vtable pointer (LDR X0, [X0, #96]
    // where X0 is zero).  Prime low-memory so it points at BootServices.
    write64(bus, BOOT_SERVICES_VPTR_ADDR, EFI_BOOT_SERVICES_ADDR);

    // Install EFI_LOADED_IMAGE_PROTOCOL
    super::protocols::install_loaded_image_protocol(bus, image_base, image_size);

    // Store image base/size for EFI stub use
    write64(bus, EFI_IMAGE_INFO_ADDR, image_base);
    write64(bus, EFI_IMAGE_INFO_ADDR + 8, image_size);

    (handle, st)
}
