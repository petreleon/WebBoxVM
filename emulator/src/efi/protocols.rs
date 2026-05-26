//! EFI Protocol structures needed by the Linux boot stub.

use crate::bus::SystemBus;
use super::encode::write64;
use super::layout::EFI_MEM_BASE;

/// EFI_LOADED_IMAGE_PROTOCOL — installed on the image handle.
/// Layout (64-bit offsets):
///   +0x00  Revision          (u32)
///   +0x04  Reserved          (u32)
///   +0x08  ParentHandle      (u64 ptr)
///   +0x10  SystemTable       (u64 ptr)
///   +0x18  DeviceHandle      (u64 ptr)
///   +0x20  FilePath          (u64 ptr)
///   +0x28  Reserved2         (u64 ptr)
///   +0x30  LoadOptionsSize   (u32)
///   +0x34  Reserved3         (u32)
///   +0x38  LoadOptions       (u64 ptr)
///   +0x40  ImageBase         (u64 ptr)
///   +0x48  ImageSize         (u64)
///   +0x50  ImageCodeType     (u32)
///   +0x54  ImageDataType     (u32)
///   +0x58  Unload            (u64 ptr)  → EFI_IMAGE_UNLOAD
const LIP_OFFSET: u64 = EFI_MEM_BASE + 0x8000;

/// GUID for EFI_LOADED_IMAGE_PROTOCOL
pub const LOADED_IMAGE_PROTOCOL_GUID: u128 =
    0x5B1B31A1_9562_11D2_8E3F_00A0C969723B;

/// Install EFI_LOADED_IMAGE_PROTOCOL on the image handle.
/// Returns the address of the installed protocol structure.
pub fn install_loaded_image_protocol(
    bus: &mut SystemBus,
    image_base: u64,
    image_size: u64,
) -> u64 {
    let base = LIP_OFFSET;

    write64(bus, base + 0x00, 0x1000); // Revision = 0x1000
    write64(bus, base + 0x08, 0);     // ParentHandle
    write64(bus, base + 0x10, 0);     // SystemTable (set later)
    write64(bus, base + 0x18, 0);     // DeviceHandle
    write64(bus, base + 0x20, 0);     // FilePath
    write64(bus, base + 0x28, 0);     // Reserved
    write64(bus, base + 0x30, 0);     // LoadOptionsSize = 0 (no cmdline)
    write64(bus, base + 0x38, 0);     // LoadOptions = null
    write64(bus, base + 0x40, image_base); // ImageBase
    write64(bus, base + 0x48, image_size); // ImageSize
    write64(bus, base + 0x50, 0);     // ImageCodeType
    write64(bus, base + 0x58, 0);     // Unload = null

    base
}

/// Get the address of the installed EFI_LOADED_IMAGE_PROTOCOL.
pub fn loaded_image_protocol_addr() -> u64 {
    LIP_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::SystemBus;

    #[test]
    fn lip_installed() {
        let mut bus = SystemBus::new();
        let addr = install_loaded_image_protocol(&mut bus, 0x4008_0000, 0x100_0000);
        assert_eq!(addr, LIP_OFFSET);
        let base = bus.read(addr + 0x40, 8).unwrap();
        assert_eq!(base, 0x4008_0000);
        let size = bus.read(addr + 0x48, 8).unwrap();
        assert_eq!(size, 0x100_0000);
    }
}
