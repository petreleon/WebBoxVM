//! EFI Protocol structures needed by the Linux boot stub.

use crate::bus::SystemBus;
use crate::constants::*;
use super::encode::write64;

// Re-export GUID constant (kept here for backward compat)
pub const LOADED_IMAGE_GUID_LO: u64 = LIP_GUID_LO;

/// Install EFI_LOADED_IMAGE_PROTOCOL (LIP) at its reserved address.
///
/// LIP layout (64-bit offsets):
/// ```text
///   +0x00  Revision          (u32)
///   +0x08  ParentHandle      (u64)
///   +0x10  SystemTable       (u64)
///   +0x18  DeviceHandle      (u64)
///   +0x20  FilePath          (u64)
///   +0x28  Reserved2         (u64)
///   +0x30  LoadOptionsSize   (u32)
///   +0x38  LoadOptions       (u64)
///   +0x40  ImageBase         (u64) ← filled with actual kernel image base
///   +0x48  ImageSize         (u64) ← filled with actual kernel image size
///   +0x50  ImageCodeType     (u32)
///   +0x58  Unload            (u64)
/// ```
pub fn install_loaded_image_protocol(
    bus: &mut SystemBus,
    image_base: u64,
    image_size: u64,
) -> u64 {
    let base = LIP_STRUCT_ADDR;

    write64(bus, base + 0x00, LIP_REVISION); // Revision
    write64(bus, base + 0x08, 0);            // ParentHandle
    write64(bus, base + 0x10, 0);            // SystemTable
    write64(bus, base + 0x18, 0);            // DeviceHandle
    write64(bus, base + 0x20, 0);            // FilePath
    write64(bus, base + 0x28, 0);            // Reserved
    write64(bus, base + 0x30, 0);            // LoadOptionsSize
    write64(bus, base + 0x38, 0);            // LoadOptions
    write64(bus, base + 0x40, image_base);   // ImageBase
    write64(bus, base + 0x48, image_size);   // ImageSize
    write64(bus, base + 0x50, 0);            // ImageCodeType
    write64(bus, base + 0x58, 0);            // Unload

    base
}

/// Returns the address of the installed Loaded Image Protocol.
pub fn loaded_image_protocol_addr() -> u64 {
    LIP_STRUCT_ADDR
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::SystemBus;

    #[test]
    fn lip_installed() {
        let mut bus = SystemBus::new();
        let addr = install_loaded_image_protocol(&mut bus, KERNEL_LOAD_ADDR, 0x100_0000);
        assert_eq!(addr, LIP_STRUCT_ADDR);
        let base = bus.read(addr + 0x40, 8).unwrap();
        assert_eq!(base, KERNEL_LOAD_ADDR);
        let size = bus.read(addr + 0x48, 8).unwrap();
        assert_eq!(size, 0x100_0000);
    }
}
