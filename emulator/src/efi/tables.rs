use crate::bus::SystemBus;
use super::encode::{write64, write32, write_success_trampoline};
use super::layout::*;

pub fn setup_efi_tables(bus: &mut SystemBus, image_base: u64, image_size: u64) -> (u64, u64) {
    let handle = EFI_HANDLE_ADDR;
    write64(bus, handle, 0xDEAD_BEEF_CAFE_BABE);
    write64(bus, EFI_ST_PTR_ADDR, EFI_SYSTEM_TABLE);

    let st = EFI_SYSTEM_TABLE;
    write64(bus, st + 0x00, 0x5453_5953_2049_4249);
    write32(bus, st + 0x08, 0x0002_001E);
    write32(bus, st + 0x0C, 0x78);
    write32(bus, st + 0x10, 0);
    write32(bus, st + 0x14, 0);
    write64(bus, st + 0x18, 0);
    write32(bus, st + 0x20, 0);
    write64(bus, st + 0x24, 0);
    write64(bus, st + 0x2C, 0);
    write64(bus, st + 0x34, 0);
    write64(bus, st + 0x3C, 0);
    write64(bus, st + 0x44, 0);
    write64(bus, st + 0x4C, 0);
    write64(bus, st + 0x54, EFI_RUNTIME_SERVICES);
    write64(bus, st + 0x5C, EFI_BOOT_SERVICES);
    write64(bus, st + 0x64, 0);
    write64(bus, st + 0x6C, 0);

    let rt_offsets = [
        0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70, 0x78, 0x80,
    ];
    for (i, off) in rt_offsets.iter().enumerate() {
        let tp = EFI_SERVICE_TRAMPOLINES + (i as u64) * TRAMPOLINE_STRIDE;
        let ptr = write_success_trampoline(bus, tp, EFI_SUCCESS);
        write64(bus, EFI_RUNTIME_SERVICES + off, ptr);
    }

    let bs_offsets = [
        0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70, 0x78,
        0x80, 0x88, 0x90, 0x98, 0xA0, 0xA8, 0xB0, 0xB8, 0xC0, 0xC8, 0xD0, 0xD8, 0xE0,
        0xE8, 0xF0, 0xF8, 0x100, 0x108, 0x110, 0x118, 0x120, 0x128, 0x130, 0x138, 0x140,
        0x148, 0x150, 0x158, 0x160, 0x168, 0x170,
    ];
    for (i, off) in bs_offsets.iter().enumerate() {
        let tp = EFI_SERVICE_TRAMPOLINES + (256 + i as u64) * TRAMPOLINE_STRIDE;
        let ptr = write_success_trampoline(bus, tp, EFI_SUCCESS);
        write64(bus, EFI_BOOT_SERVICES + off, ptr);
    }

    write64(bus, EFI_MEM_BASE + 0xFF00, image_base);
    write64(bus, EFI_MEM_BASE + 0xFF08, image_size);
    (handle, st)
}
