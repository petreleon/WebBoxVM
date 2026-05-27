use crate::bus::SystemBus;
use super::encode::{movz_x, movk_x, write64, write32};
use super::layout::*;

// ── Custom trampolines ──────────────────────────────────────────

const GET_MEMMAP_INST: [u32; 3] = [
    0xD280_0000, // MOVZ X0, #0
    0xD65F_03C0, // RET
    0xD503_201F, // no-op pad
];

const LOCATE_PROTO_INST: [u32; 3] = [
    0xD280_01C0, // MOVZ X0, #0xE
    0xF2BF_FFC0, // MOVK X0, #0x8000, LSL #48
    0xD65F_03C0, // RET
];

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

pub fn setup_efi_tables(bus: &mut SystemBus, image_base: u64, image_size: u64) -> (u64, u64) {
    let handle = EFI_HANDLE_ADDR;
    // The EFI stub dereferences ImageHandle (e.g. LDR X0,[X0,#96]).
    // Point it at the kernel region base so the load succeeds but the
    // actual LoadedImageProtocol is still returned by HandleProtocol.
    // The EFI stub dereferences ImageHandle (e.g. LDR X0,[X0,#96]).
    // Point it at the kernel region base so the read succeeds.
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
    write64(bus, st + 0x28, 0);                     // ConsoleInHandle
    write64(bus, st + 0x30, 0);                     // ConIn
    write64(bus, st + 0x38, 0);                     // ConsoleOutHandle
    write64(bus, st + 0x40, 0);                     // ConOut
    write64(bus, st + 0x48, 0);                     // StandardErrorHandle
    write64(bus, st + 0x50, 0);                     // StdErr
    write64(bus, st + 0x58, EFI_RUNTIME_SERVICES);   // RuntimeServices
    write64(bus, st + 0x60, EFI_BOOT_SERVICES);      // BootServices
    write64(bus, st + 0x68, 0);                      // NumberOfTableEntries
    write64(bus, st + 0x70, 0);                      // ConfigurationTable

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
    // Slots 0..41 cover offsets 0x18..0x170 (step 8)
    let boot_slots = [
        (0x18, None), (0x20, None), (0x28, None), (0x30, None),
        (0x38, None),
        (0x40, None), (0x48, None), (0x50, None), (0x58, None),
        (0x60, None), (0x68, None), (0x70, None), (0x78, None),
        (0x80, None), (0x88, None), (0x90, None),
        (0x98, None),
        (0xA0, None), (0xA8, None), (0xB0, None), (0xB8, None),
        (0xC0, None), (0xC8, None), (0xD0, None), (0xD8, None),
        (0xE0, None), (0xE8, None), (0xF0, None), (0xF8, None),
        (0x100, None), (0x108, None), (0x110, None), (0x118, None),
        (0x120, None), (0x128, None), (0x130, None), (0x138, None),
        (0x140, Some(&LOCATE_PROTO_INST[..])), (0x148, None), (0x150, None), (0x158, None),
        (0x160, None), (0x168, None), (0x170, None),
    ];

    for (i, &(off, maybe_custom)) in boot_slots.iter().enumerate() {
        let tp = EFI_SERVICE_TRAMPOLINES + (256 + i as u64) * TRAMPOLINE_STRIDE;
        if let Some(custom) = maybe_custom {
            write_trampoline(bus, tp, custom);
            write64(bus, EFI_BOOT_SERVICES + off, tp);
        } else {
            let ptr = super::encode::write_success_trampoline(bus, tp, EFI_SUCCESS);
            write64(bus, EFI_BOOT_SERVICES + off, ptr);
        }
    }

    // Fix AllocatePool to use the bump allocator.
    // AllocatePool is boot slot i=5 (offset 0x40). Use its trampoline slot.
    let pool_tp = EFI_SERVICE_TRAMPOLINES + (256 + 5) * TRAMPOLINE_STRIDE;
    let bump = bump_allocator_trampoline(POOL_HEAD, POOL_BASE);
    write_trampoline(bus, pool_tp, &bump);
    write64(bus, EFI_BOOT_SERVICES + 0x40, pool_tp);
    // Prime the bump head so the first allocation starts at POOL_BASE.
    write64(bus, POOL_HEAD, POOL_BASE);

    // Fix GetMemoryMap (offset 0x38) using a dynamic trampoline at safe/empty slot 50.
    let memmap_tp = EFI_SERVICE_TRAMPOLINES + 50 * TRAMPOLINE_STRIDE;
    let memmap = [
        0xf9400005, // LDR X5, [X0]           // load *MemoryMapSize
        0xF100C0BF, // CMP X5, #48            // compare size with 48
        0x54000122, // B.HS label_fill        // if size >= 48, jump to fill
        
        // size < 48:
        movz_x(5, 48),                        // MOV X5, #48
        0xf9000005,                           // STR X5, [X0]   // *MemoryMapSize = 48
        movz_x(5, 48),                        // MOV X5, #48
        0xf9000065,                           // STR X5, [X3]   // *DescriptorSize = 48
        movz_x(5, 1),                         // MOV X5, #1
        0xb9000085,                           // STR W5, [X4]   // *DescriptorVersion = 1
        movz_x(0, 5),                         // MOV X0, #5     // EFI_BUFFER_TOO_SMALL
        movk_x(0, 3, 0x8000),                 // MOVK X0, #0x8000, LSL #48
        0x14000015,                           // B label_ret    // jump to ret
        
        // label_fill:
        movz_x(5, 7),                         // MOV X5, #7     // Type = EfiConventionalMemory
        0xb9000025,                           // STR W5, [X1]   // store Type
        movz_x(5, 0),                         // MOV X5, #0
        0xb9000425,                           // STR W5, [X1, #4] // store Pad
        movz_x(5, 0x0000),                    // MOV X5, #0x0000
        movk_x(5, 2, 0x4000),                 // MOVK X5, #0x4000, LSL #32
        0xf9000425,                           // STR X5, [X1, #8] // PhysicalStart
        0xf900083f,                           // STR XZR, [X1, #16] // VirtualStart
        movz_x(5, 0x0000),                    // MOV X5, #0x0000
        movk_x(5, 1, 0x0004),                 // MOVK X5, #4, LSL #16 // 262144 pages
        0xf9000c25,                           // STR X5, [X1, #24] // NumberOfPages
        movz_x(5, 0x000F),                    // MOV X5, #0xF   // Attribute = 0xF
        0xf9001025,                           // STR X5, [X1, #32] // Attribute
        0xf900143f,                           // STR XZR, [X1, #40] // Pad2
        
        // set outputs:
        movz_x(5, 48),                        // MOV X5, #48
        0xf9000005,                           // STR X5, [X0]   // *MemoryMapSize = 48
        movz_x(5, 17),                        // MOV X5, #17
        0xf9000045,                           // STR X5, [X2]   // *MapKey = 17
        movz_x(5, 48),                        // MOV X5, #48
        0xf9000065,                           // STR X5, [X3]   // *DescriptorSize = 48
        movz_x(5, 1),                         // MOV X5, #1
        0xb9000085,                           // STR W5, [X4]   // *DescriptorVersion = 1
        movz_x(0, 0),                         // MOV X0, #0     // EFI_SUCCESS
        
        // label_ret:
        ret(),                                // RET
    ];
    write_trampoline(bus, memmap_tp, &memmap);
    write64(bus, EFI_BOOT_SERVICES + 0x38, memmap_tp);

    // Fix HandleProtocol (offset 0x98) using a dynamic trampoline at safe/empty slot 60.
    let handle_proto_tp = EFI_SERVICE_TRAMPOLINES + 60 * TRAMPOLINE_STRIDE;
    let handle_proto = [
        0xf9400024,                              // LDR X4, [X1]
        movz_x(3, 0x31A1),                       // MOVZ X3, #0x31A1
        movk_x(3, 1, 0x5B1B),                    // MOVK X3, #0x5B1B, LSL #16
        movk_x(3, 2, 0x9562),                    // MOVK X3, #0x9562, LSL #32
        movk_x(3, 3, 0x11D2),                    // MOVK X3, #0x11D2, LSL #48
        0xCB030084,                              // SUB X4, X4, X3
        0xB50000C4,                              // CBNZ X4, label_unsupported
        movz_x(3, 0x8000),                       // MOVZ X3, #0x8000
        movk_x(3, 1, 0x8000),                    // MOVK X3, #0x8000, LSL #16
        0xf9000043,                              // STR X3, [X2]
        movz_x(0, 0),                            // MOVZ X0, #0 (EFI_SUCCESS)
        0x14000003,                              // B label_ret
        // label_unsupported:
        movz_x(0, 3),                            // MOVZ X0, #3 (EFI_UNSUPPORTED)
        movk_x(0, 3, 0x8000),                    // MOVK X0, #0x8000, LSL #48
        // label_ret:
        ret(),                                   // RET
    ];
    write_trampoline(bus, handle_proto_tp, &handle_proto);
    write64(bus, EFI_BOOT_SERVICES + 0x98, handle_proto_tp);

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
