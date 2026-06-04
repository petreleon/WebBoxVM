use super::*;

const RUNTIME_SERVICE_OFFSETS: [u64; 14] = [
    0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70, 0x78, 0x80,
];

const BOOT_SERVICE_OFFSETS: &[u64] = &[
    0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70, 0x78, 0x80, 0x88, 0x90,
    0x98, 0xA0, 0xA8, 0xB0, 0xB8, 0xC0, 0xC8, 0xD0, 0xD8, 0xE0, 0xE8, 0xF0, 0xF8, 0x100, 0x108,
    0x110, 0x118, 0x120, 0x128, 0x130, 0x138, 0x140, 0x148, 0x150, 0x158, 0x160, 0x168, 0x170,
];

pub(super) fn install_default_services(bus: &mut SystemBus) {
    install_runtime_services(bus);
    install_boot_services(bus);
}

fn install_runtime_services(bus: &mut SystemBus) {
    for (i, &off) in RUNTIME_SERVICE_OFFSETS.iter().enumerate() {
        let addr = EFI_TRAMPOLINES_ADDR + (i as u64) * TRAMPOLINE_SLOT_SIZE;
        let ptr = super::super::encode::write_success_trampoline(bus, addr, EFI_SUCCESS);
        write64(bus, EFI_RUNTIME_SERVICES_ADDR + off, ptr);
    }
}

fn install_boot_services(bus: &mut SystemBus) {
    for (i, &off) in BOOT_SERVICE_OFFSETS.iter().enumerate() {
        let addr = EFI_TRAMPOLINES_ADDR + (256 + i as u64) * TRAMPOLINE_SLOT_SIZE;
        let ptr = super::super::encode::write_success_trampoline(bus, addr, EFI_SUCCESS);
        write64(bus, EFI_BOOT_SERVICES_ADDR + off, ptr);
    }
}
