use crate::bus::SystemBus;
use super::encode::{write64, write32, write_success_trampoline};
use super::layout::*;

// ── Custom trampolines ──────────────────────────────────────────
const HANDLE_PROTO_INST: [u32; 5] = [
    0xD290_0003, // MOVZ X3, #0x8000
    0xF2B0_0003, // MOVK X3, #0x8000, LSL #16
    0xF900_0043, // STR  X3, [X2]
    0xD280_0000, // MOVZ X0, #0
    0xD65F_03C0, // RET
];

// GetMemoryMap: return EFI_SUCCESS and set *MemoryMapSize = 0
const GET_MEMMAP_INST: [u32; 3] = [
    0xD280_0000, // MOVZ X0, #0
    0xD65F_03C0, // RET
    // no-op pad
    0xD503_201F,
];

/// Write a sequence of ARM64 instruction words to a trampoline slot.
fn write_trampoline(bus: &mut SystemBus, addr: u64, insts: &[u32]) {
    for (i, &inst) in insts.iter().enumerate() {
        write32(bus, addr + (i as u64 * 4), inst);
    }
}

pub fn setup_efi_tables(bus: &mut SystemBus, image_base: u64, image_size: u64) -> (u64, u64) {
    let handle = EFI_HANDLE_ADDR;
    write64(bus, handle, 0xDEAD_BEEF_CAFE_BABE);
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
        let ptr = write_success_trampoline(bus, tp, EFI_SUCCESS);
        write64(bus, EFI_RUNTIME_SERVICES + off, ptr);
    }

    // ── Boot Services ──
    let bs_slots: &[(u64, Option<&[u32]>)] = &[
        (0x18, None), (0x20, None), (0x28, None), (0x30, None),
        (0x38, Some(&GET_MEMMAP_INST)),
        (0x40, None), (0x48, None), (0x50, None), (0x58, None),
        (0x60, None), (0x68, None), (0x70, None), (0x78, None),
        (0x80, None), (0x88, None), (0x90, None),
        (0x98, Some(&HANDLE_PROTO_INST)),
        (0xA0, None), (0xA8, None), (0xB0, None), (0xB8, None),
        (0xC0, None), (0xC8, None), (0xD0, None), (0xD8, None),
        (0xE0, None), (0xE8, None), (0xF0, None), (0xF8, None),
        (0x100, None), (0x108, None), (0x110, None), (0x118, None),
        (0x120, None), (0x128, None), (0x130, None), (0x138, None),
        (0x140, None), (0x148, None), (0x150, None), (0x158, None),
        (0x160, None), (0x168, None), (0x170, None),
    ];

    for (i, &(off, maybe_custom)) in bs_slots.iter().enumerate() {
        let tp = EFI_SERVICE_TRAMPOLINES + (256 + i as u64) * TRAMPOLINE_STRIDE;
        if let Some(custom) = maybe_custom {
            write_trampoline(bus, tp, custom);
            write64(bus, EFI_BOOT_SERVICES + off, tp);
        } else {
            let ptr = write_success_trampoline(bus, tp, EFI_SUCCESS);
            write64(bus, EFI_BOOT_SERVICES + off, ptr);
        }
    }

    // Install EFI_LOADED_IMAGE_PROTOCOL
    super::protocols::install_loaded_image_protocol(bus, image_base, image_size);

    write64(bus, EFI_MEM_BASE + 0xFF00, image_base);
    write64(bus, EFI_MEM_BASE + 0xFF08, image_size);
    (handle, st)
}
