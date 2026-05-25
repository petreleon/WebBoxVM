//! Minimal UEFI runtime for booting PE/EFI kernels.

pub mod encode;
pub mod layout;
pub mod tables;

pub use layout::{is_efi_addr, EFI_MEM_BASE, EFI_MEM_SIZE, EFI_HANDLE_ADDR, EFI_SYSTEM_TABLE};
pub use tables::setup_efi_tables;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::SystemBus;

    #[test]
    fn efi_tables_initialised() {
        let mut bus = SystemBus::new();
        let (handle, st) = tables::setup_efi_tables(&mut bus, 0x4000_0000, 0x100_0000);
        assert!(handle != 0);
        let sig = bus.read(st, 8).unwrap();
        assert_eq!(sig, 0x5453_5953_2049_4249);
    }
}
