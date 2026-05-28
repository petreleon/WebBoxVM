use crate::bus::SystemBus;
use super::encode::{movz_x, movk_x, write64, write32};
use super::layout::*;
use super::protocols::{loaded_image_protocol_addr, LOADED_IMAGE_GUID_LO};


// ── Simple "return EFI_SUCCESS" trampoline ────────────────────────
fn ret() -> u32 { 0xD65F_03C0 }

// Pool base inside RAM. Must remain below 0x43FF_F000 (our SP).
const POOL_BASE: u64 = 0x43A0_A000;
// Bump-head pointer lives inside the EFI scratch area (16 MiB above EFI_MEM_BASE).
const POOL_HEAD: u64 = EFI_MEM_BASE + 0x0FFF8;

fn write_trampoline(bus: &mut SystemBus, addr: u64, insts: &[u32]) {
    for (i, &inst) in insts.iter().enumerate() {
        write32(bus, addr + (i as u64 * 4), inst);
    }
}

// Encode the bump-allocator trampoline at runtime so the address constants are
// guaranteed correct (no risk of hand-typos).
fn bump_allocator_trampoline(head: u64, _base: u64) -> [u32; 8] {
    [
        movz_x(4, (head & 0xFFFF) as u16),
        movk_x(4, 1, ((head >> 16) & 0xFFFF) as u16),
        0xF9400085,              // LDR  X5, [X4]      // read current pool head
        0x8B0100A0,              // ADD  X0, X5, X1    // X0 = head + size (X1=Size)
        0xF9000080,              // STR  X0, [X4]      // update pool head
        0xF9000045,              // STR  X5, [X2]      // *Buffer = old head (X2=**Buffer)
        movz_x(0, 0),            // MOVZ X0, #0      // EFI_SUCCESS
        ret(),
    ]
}

/// Build a MOVZ/MOVK sequence for a 64-bit constant into register d (up to 4 instructions).
/// Returns number of instructions written.
fn encode_mov64(insts: &mut Vec<u32>, d: u8, val: u64) {
    insts.push(movz_x(d, (val & 0xFFFF) as u16));
    if val >> 16 != 0 {
        insts.push(movk_x(d, 1, ((val >> 16) & 0xFFFF) as u16));
    }
    if val >> 32 != 0 {
        insts.push(movk_x(d, 2, ((val >> 32) & 0xFFFF) as u16));
    }
    if val >> 48 != 0 {
        insts.push(movk_x(d, 3, ((val >> 48) & 0xFFFF) as u16));
    }
}

/// Build the GetMemoryMap large trampoline.
/// Signature: GetMemoryMap(MemoryMapSize*, MemoryMap*, MapKey*, DescriptorSize*, DescriptorVersion*)
///   X0 = *MemoryMapSize
///   X1 = MemoryMap  (buffer pointer)
///   X2 = *MapKey
///   X3 = *DescriptorSize
///   X4 = *DescriptorVersion
///
/// Strategy: If the buffer is big enough for 1 descriptor (48 bytes), fill 1 entry.
/// Otherwise return EFI_BUFFER_TOO_SMALL with needed size.
fn build_get_memory_map_trampoline() -> Vec<u32> {
    // This is a 64-byte+ trampoline placed in EFI_LARGE_CODE area.
    // Using registers: X5=scratch, X6=loaded *MemoryMapSize
    let mut v = Vec::new();

    // X6 = *MemoryMapSize (current value)
    v.push(0xf9400006); // LDR X6, [X0]
    // Compare with 48 (one EFI_MEMORY_DESCRIPTOR)
    v.push(0xF100C0DF); // CMP X6, #48   (SUBS XZR, X6, #48)
    // If X6 >= 48: skip to fill
    // B.HS label_fill  (condition HS = unsigned higher or same, cond=2)
    v.push(0x54000002); // B.HS placeholder
    // label_too_small (if X6 < 48):
    encode_mov64(&mut v, 5, 48); // MOV X5, #48
    v.push(0xf9000005); // STR X5, [X0]   // *MemoryMapSize = 48
    v.push(0xf9000065); // STR X5, [X3]   // *DescriptorSize = 48
    encode_mov64(&mut v, 5, 1); // MOV X5, #1
    v.push(0xb9000085); // STR W5, [X4]   // *DescriptorVersion = 1
    encode_mov64(&mut v, 0, 0x8000_0000_0000_0005u64); // EFI_BUFFER_TOO_SMALL
    v.push(ret()); // RET

    let too_small_len = v.len() as i32; // instructions before fill

    // label_fill: fill one EFI_MEMORY_DESCRIPTOR at X1
    // Type = EfiConventionalMemory (7)
    encode_mov64(&mut v, 5, 7);
    v.push(0xb9000025); // STR W5, [X1]       // Type
    v.push(0xb9000425); // STR W5, [X1, #4]   // Pad (same val, doesn't matter)
    // Actually Pad should be 0
    v.push(0xb900043f); // STR WZR, [X1, #4]  // Pad = 0  (overwrites)
    // PhysicalStart = 0x4000_0000
    encode_mov64(&mut v, 5, 0x4000_0000u64);
    v.push(0xf9000425); // STR X5, [X1, #8]   // PhysicalStart
    v.push(0xf900083f); // STR XZR, [X1, #16] // VirtualStart = 0
    // NumberOfPages = 0x40000 (1 GB / 4KB)
    encode_mov64(&mut v, 5, 0x40000u64);
    v.push(0xf9000c25); // STR X5, [X1, #24]  // NumberOfPages
    // Attributes = EFI_MEMORY_WB (0xF)
    encode_mov64(&mut v, 5, 0xFu64);
    v.push(0xf9001025); // STR X5, [X1, #32]  // Attribute
    v.push(0xf9001437); // STR X23, [X1, #40] // Pad2 (use X23 which might be 0 - actually use XZR)
    v.push(0xf900143f); // STR XZR, [X1, #40] // Pad2 = 0

    // Set outputs
    encode_mov64(&mut v, 5, 48);
    v.push(0xf9000005); // STR X5, [X0]   // *MemoryMapSize = 48
    encode_mov64(&mut v, 5, 17);
    v.push(0xf9000045); // STR X5, [X2]   // *MapKey = 17
    encode_mov64(&mut v, 5, 48);
    v.push(0xf9000065); // STR X5, [X3]   // *DescriptorSize = 48
    encode_mov64(&mut v, 5, 1);
    v.push(0xb9000085); // STR W5, [X4]   // *DescriptorVersion = 1
    v.push(movz_x(0, 0)); // MOV X0, #0  // EFI_SUCCESS
    v.push(ret());

    // Now fix the B.HS branch: it's at index 2, needs to skip `too_small_len - 3` instructions
    // Branch at index 2: if HS (X6>=48) jump to label_fill
    // label_fill starts at index `too_small_len`
    // B.HS target = PC + offset = instr[2] + (too_small_len - 2) * 4
    // imm19 = (too_small_len - 2)
    let branch_offset = (too_small_len - 2) as u32; // positive, skip too_small body
    // B.cond: bits[31:24]=0x54, bits[23:5]=imm19 (signed), bits[4:0]=cond
    // cond HS = 0b0010 = 2
    let bcond_hs = 0x54000002u32 | ((branch_offset & 0x7FFFF) << 5);
    v[2] = bcond_hs;

    println!("GetMemoryMap trampoline instructions:");
    for (i, inst) in v.iter().enumerate() {
        println!("  inst[{}] = 0x{:08x}", i, inst);
    }

    v
}

/// Build a HandleProtocol / OpenProtocol large trampoline.
/// 
/// HandleProtocol:  X0=Handle, X1=Protocol (GUID*), X2=Interface**
/// OpenProtocol:    X0=Handle, X1=Protocol (GUID*), X2=Interface**, X3=Agent, X4=Controller, X5=Attributes
///
/// Checks if Protocol GUID matches EFI_LOADED_IMAGE_PROTOCOL.
/// If yes: *Interface = LIP_OFFSET, return EFI_SUCCESS.
/// If no: *Interface = NULL (if X2 != 0), return EFI_NOT_FOUND.
fn build_handle_protocol_trampoline(lip_addr: u64) -> Vec<u32> {
    // EFI_LOADED_IMAGE_PROTOCOL GUID in memory layout:
    //   First 8 bytes as little-endian u64: 0x11D2_9562_5B1B_31A1
    let guid_lo: u64 = LOADED_IMAGE_GUID_LO;

    let mut v = Vec::new();

    // Load first 8 bytes of GUID at [X1]

    v.push(0xf9400024); // LDR X4, [X1]
    // Build expected GUID low bits into X3
    encode_mov64(&mut v, 3, guid_lo);
    // Compare
    v.push(0xEB030084); // SUBS X4, X4, X3  (sets flags: Z if equal)
    // If not zero: jump to not_found
    // CBNZ X4, label_not_found — we'll patch the offset
    let cbnz_idx = v.len();
    v.push(0xB5000004); // placeholder CBNZ X4, +4 instructions ahead (patch below)

    // GUID matches — return LIP_OFFSET
    // *Interface (X2) = lip_addr
    encode_mov64(&mut v, 3, lip_addr);
    v.push(0xf9000043); // STR X3, [X2]
    v.push(movz_x(0, 0)); // EFI_SUCCESS
    v.push(ret());

    let not_found_idx = v.len();

    // label_not_found: *Interface = NULL (if X2 != 0), return EFI_NOT_FOUND
    // STR XZR, [X2]  — safe only if X2 != 0; LocateProtocol always provides X2
    v.push(0xF900005F); // STR XZR, [X2]  // *Interface = NULL
    // EFI_NOT_FOUND = 0x800000000000000E
    encode_mov64(&mut v, 0, 0x8000_0000_0000_000Eu64);
    v.push(ret());

    // Patch CBNZ X4: offset in instructions from cbnz_idx to not_found_idx
    let offset = (not_found_idx as i32 - cbnz_idx as i32) as u32;
    // CBNZ Xt: bits[31:24]=0xB5, bits[23:5]=imm19 (signed, in instructions), bits[4:0]=Rt
    v[cbnz_idx] = 0xB5000004u32 | ((offset & 0x7FFFF) << 5);

    v
}

pub fn setup_efi_tables(bus: &mut SystemBus, image_base: u64, image_size: u64, dtb_addr: u64) -> (u64, u64) {
    let handle = EFI_HANDLE_ADDR;
    write64(bus, handle, 0x1_0000);
    write64(bus, EFI_ST_PTR_ADDR, EFI_SYSTEM_TABLE);

    let st = EFI_SYSTEM_TABLE;
    write64(bus, st + 0x00, 0x5453_5953_2049_4249); // Signature
    write32(bus, st + 0x08, 0x0002_001E);           // Revision
    write32(bus, st + 0x0C, 0x78);                  // HeaderSize
    write32(bus, st + 0x10, 0);                     // CRC32
    write32(bus, st + 0x14, 0);                     // Reserved
    write64(bus, st + 0x18, 0);                     // FirmwareVendor
    write32(bus, st + 0x20, 0);                     // FirmwareRevision
    write32(bus, st + 0x24, 0);                     // PAD to 8-byte alignment
    let con_out_struct = EFI_MEM_BASE + 0x6000;
    let con_out_handle = EFI_MEM_BASE + 0x5000;
    write64(bus, st + 0x28, 0);                     // ConsoleInHandle
    write64(bus, st + 0x30, 0);                     // ConIn
    write64(bus, st + 0x38, con_out_handle);        // ConsoleOutHandle
    write64(bus, st + 0x40, con_out_struct);        // ConOut
    write64(bus, st + 0x48, con_out_handle);        // StandardErrorHandle
    write64(bus, st + 0x50, con_out_struct);        // StdErr
    write64(bus, st + 0x58, EFI_RUNTIME_SERVICES);   // RuntimeServices
    write64(bus, st + 0x60, EFI_BOOT_SERVICES);      // BootServices

    // Register FDT configuration table
    let config_table_addr = EFI_MEM_BASE + 0x9000;
    // FDT GUID: b1b621d5-f19c-41a5-830b-d9152c69aae0
    write64(bus, config_table_addr + 0, 0x41a5_f19c_b1b6_21d5);
    write64(bus, config_table_addr + 8, 0xe0aa_692c_15d9_0b83);
    write64(bus, config_table_addr + 16, dtb_addr);

    write64(bus, st + 0x68, 1);                      // NumberOfTableEntries
    write64(bus, st + 0x70, config_table_addr);      // ConfigurationTable

    // ── Runtime Services ──
    let rt_offsets = [
        0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70, 0x78, 0x80,
    ];
    for (i, off) in rt_offsets.iter().enumerate() {
        let tp = EFI_SERVICE_TRAMPOLINES + (i as u64) * TRAMPOLINE_STRIDE;
        let ptr = super::encode::write_success_trampoline(bus, tp, EFI_SUCCESS);
        write64(bus, EFI_RUNTIME_SERVICES + off, ptr);
    }

    // ── Boot Services ──
    // Offsets 0x18..0x170 (every 8 bytes), 43 entries total.
    // We start numbering boot slots at 256 to not clash with runtime slots.
    //
    // UEFI spec EFI_BOOT_SERVICES layout (64-bit):
    //   0x18 RaiseTPL         0x20 RestoreTPL
    //   0x28 AllocatePages    0x30 FreePages
    //   0x38 GetMemoryMap     0x40 AllocatePool      0x48 FreePool
    //   0x50 CreateEvent      0x58 SetTimer          0x60 WaitForEvent
    //   0x68 SignalEvent      0x70 CloseEvent        0x78 CheckEvent
    //   0x80 InstallProtoIface 0x88 ReinstallProto   0x90 UninstallProto
    //   0x98 HandleProtocol   0xA0 Reserved          0xA8 RegisterProtoNotify
    //   0xB0 LocateHandle     0xB8 LocateDevicePath  0xC0 InstallConfigTable
    //   0xC8 LoadImage        0xD0 StartImage        0xD8 Exit
    //   0xE0 UnloadImage      0xE8 ExitBootServices  0xF0 GetNextMonotonicCount
    //   0xF8 Stall            0x100 SetWatchdogTimer
    //   0x108 ConnectController 0x110 DisconnectController
    //   0x118 OpenProtocol    0x120 CloseProtocol    0x128 OpenProtocolInformation
    //   0x130 ProtocolsPerHandle 0x138 LocateHandleBuffer
    //   0x140 LocateProtocol  0x148 InstallMultipleProtoIfaces
    //   0x150 UninstallMultipleProtoIfaces
    //   0x158 CalculateCrc32  0x160 CopyMem          0x168 SetMem
    //   0x170 CreateEventEx
    let boot_offsets: &[u64] = &[
        0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60,
        0x68, 0x70, 0x78, 0x80, 0x88, 0x90, 0x98, 0xA0, 0xA8, 0xB0,
        0xB8, 0xC0, 0xC8, 0xD0, 0xD8, 0xE0, 0xE8, 0xF0, 0xF8, 0x100,
        0x108, 0x110, 0x118, 0x120, 0x128, 0x130, 0x138, 0x140, 0x148, 0x150,
        0x158, 0x160, 0x168, 0x170,
    ];
    for (i, &off) in boot_offsets.iter().enumerate() {
        let tp = EFI_SERVICE_TRAMPOLINES + (256 + i as u64) * TRAMPOLINE_STRIDE;
        let ptr = super::encode::write_success_trampoline(bus, tp, EFI_SUCCESS);
        write64(bus, EFI_BOOT_SERVICES + off, ptr);
    }

    // ── Specialised large trampolines (placed in EFI_LARGE_CODE area) ──
    // Each block is LARGE_CODE_STRIDE bytes = 512 bytes max.

    // Block 0: AllocatePool (0x40) — bump allocator (8 instructions = 32 bytes, fits in stride)
    let pool_tp = EFI_LARGE_CODE + 0 * LARGE_CODE_STRIDE;
    let bump = bump_allocator_trampoline(POOL_HEAD, POOL_BASE);
    write_trampoline(bus, pool_tp, &bump);
    write64(bus, EFI_BOOT_SERVICES + 0x40, pool_tp);
    // Prime the bump head so the first allocation starts at POOL_BASE.
    write64(bus, POOL_HEAD, POOL_BASE);

    // Block 1: GetMemoryMap (0x38)
    let memmap_tp = EFI_LARGE_CODE + 1 * LARGE_CODE_STRIDE;
    let memmap = build_get_memory_map_trampoline();
    assert!(memmap.len() * 4 <= LARGE_CODE_STRIDE as usize,
            "GetMemoryMap trampoline too large: {} instructions", memmap.len());
    write_trampoline(bus, memmap_tp, &memmap);
    write64(bus, EFI_BOOT_SERVICES + 0x38, memmap_tp);

    // Block 2: HandleProtocol (0x98)
    let lip_addr = loaded_image_protocol_addr();
    let hp = build_handle_protocol_trampoline(lip_addr);
    let hp_tp = EFI_LARGE_CODE + 2 * LARGE_CODE_STRIDE;
    assert!(hp.len() * 4 <= LARGE_CODE_STRIDE as usize,
            "HandleProtocol trampoline too large: {} instructions", hp.len());
    write_trampoline(bus, hp_tp, &hp);
    write64(bus, EFI_BOOT_SERVICES + 0x98, hp_tp);

    // Block 3: OpenProtocol (0x118) — same logic as HandleProtocol
    // OpenProtocol: X0=Handle, X1=Protocol*, X2=Interface**, X3=Agent, X4=Controller, X5=Attrs
    // We can reuse the same trampoline since args X1 and X2 are in the same positions!
    let op_tp = EFI_LARGE_CODE + 3 * LARGE_CODE_STRIDE;
    write_trampoline(bus, op_tp, &hp); // same code as HandleProtocol
    write64(bus, EFI_BOOT_SERVICES + 0x118, op_tp);

    // Block 4: LocateProtocol (0x140) — return EFI_NOT_FOUND, *Interface=NULL
    // This makes optional protocols gracefully absent.
    let lp = locate_protocol_trampoline();
    let lp_tp = EFI_LARGE_CODE + 4 * LARGE_CODE_STRIDE;
    write_trampoline(bus, lp_tp, &lp);
    write64(bus, EFI_BOOT_SERVICES + 0x140, lp_tp);

    // ── ConOut / StdErr ──
    let reset_tp = EFI_LARGE_CODE + 5 * LARGE_CODE_STRIDE;
    super::encode::write_success_trampoline(bus, reset_tp, EFI_SUCCESS);
    write64(bus, con_out_struct + 0x00, reset_tp);

    let output_string_tp = EFI_LARGE_CODE + 6 * LARGE_CODE_STRIDE;
    let output_string_insts = [
        0x79400022, // LDRH W2, [X1]
        0x350000a2, // CBZ W2, #20 (jumps to MOVZ X0, #0)
        0xD2A12003, // MOVZ X3, #0x0900, LSL #16
        0x380000a2, // STRB W2, [X3]
        0x91000821, // ADD X1, X1, #2
        0x17FFFFFB, // B -20 (jumps back to 0)
        0xD2800000, // MOVZ X0, #0
        0xD65F03C0, // RET
    ];
    write_trampoline(bus, output_string_tp, &output_string_insts);
    write64(bus, con_out_struct + 0x08, output_string_tp);

    // Block 7: CopyMem (0x160)
    // X0 = Dest, X1 = Src, X2 = Length
    let copymem_tp = EFI_LARGE_CODE + 7 * LARGE_CODE_STRIDE;
    let copymem_insts = [
        0xB40000A2, // CBZ X2, #20 (done)
        0x38401823, // LDRB W3, [X1], #1
        0x38001803, // STRB W3, [X0], #1
        0xF1000442, // SUBS X2, X2, #1
        0x17FFFFFC, // B -16 (Loop)
        0xD2800000, // MOVZ X0, #0 (EFI_SUCCESS)
        0xD65F03C0, // RET
    ];
    write_trampoline(bus, copymem_tp, &copymem_insts);
    write64(bus, EFI_BOOT_SERVICES + 0x160, copymem_tp);

    // Block 8: SetMem (0x168)
    // X0 = Buffer, X1 = Size, X2 = Value
    let setmem_tp = EFI_LARGE_CODE + 8 * LARGE_CODE_STRIDE;
    let setmem_insts = [
        0xB4000081, // CBZ X1, #16 (done)
        0x38001802, // STRB W2, [X0], #1
        0xF1000421, // SUBS X1, X1, #1
        0x17FFFFFD, // B -12 (Loop)
        0xD2800000, // MOVZ X0, #0 (EFI_SUCCESS)
        0xD65F03C0, // RET
    ];
    write_trampoline(bus, setmem_tp, &setmem_insts);
    write64(bus, EFI_BOOT_SERVICES + 0x168, setmem_tp);

    // Block 9: AllocatePages (0x28)
    let alloc_pages_tp = EFI_LARGE_CODE + 9 * LARGE_CODE_STRIDE;
    super::encode::write_success_trampoline(bus, alloc_pages_tp, EFI_SUCCESS);
    write64(bus, EFI_BOOT_SERVICES + 0x28, alloc_pages_tp);

    // Block 10: FreePages (0x30)
    let freepages_tp = EFI_LARGE_CODE + 10 * LARGE_CODE_STRIDE;
    super::encode::write_success_trampoline(bus, freepages_tp, EFI_SUCCESS);
    write64(bus, EFI_BOOT_SERVICES + 0x30, freepages_tp);

    // The EFI stub dereferences a NULL vtable pointer (LDR X0, [X0, #96]
    // where X0 is zero).  To keep that dispatch working, prime the
    // low-memory slot at 0x60 so it points at the BootServices table.
    write64(bus, 0x60, EFI_BOOT_SERVICES);

    // Install EFI_LOADED_IMAGE_PROTOCOL
    super::protocols::install_loaded_image_protocol(bus, image_base, image_size);

    write64(bus, EFI_MEM_BASE + 0xFF00, image_base);
    write64(bus, EFI_MEM_BASE + 0xFF08, image_size);
    (handle, st)
}

/// LocateProtocol: return EFI_NOT_FOUND and set *Interface = NULL.
/// X0=Protocol*, X1=Registration, X2=Interface**
fn locate_protocol_trampoline() -> Vec<u32> {
    let mut v = Vec::new();
    // *Interface = NULL (X2 may or may not be 0, guard with CBZ)
    v.push(0xB4000042); // CBZ X2, skip_store (2 instructions ahead)
    v.push(0xF900005F); // STR XZR, [X2]
    // EFI_NOT_FOUND = 0x800000000000000E
    encode_mov64(&mut v, 0, 0x8000_0000_0000_000Eu64);
    v.push(ret());
    v
}
