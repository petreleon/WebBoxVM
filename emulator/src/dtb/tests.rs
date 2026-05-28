use super::*;
use crate::bus::SystemBus;

#[test]
fn dtb_magic_and_size() {
    let dtb = build_dtb(0x4000_0000, 0x4000_0000, None, None, None);
    assert!(dtb.len() >= 40);
    let magic = u32::from_be_bytes([dtb[0], dtb[1], dtb[2], dtb[3]]);
    assert_eq!(magic, 0xd00dfeed);
    let totalsize = u32::from_be_bytes([dtb[4], dtb[5], dtb[6], dtb[7]]);
    assert_eq!(totalsize as usize, dtb.len());
}

#[test]
fn dtb_with_initrd() {
    let dtb = build_dtb(
        0x4000_0000,
        0x4000_0000,
        Some(0x4200_0000),
        Some(0x4300_0000),
        Some("console=ttyAMA0"),
    );
    assert!(dtb.len() > 40);
    // Verify we can load it into memory
    let mut bus = SystemBus::new();
    load_dtb(&mut bus, 0x4800_0000, &dtb);
    let magic = u32::from_be_bytes([
        bus.mem.read(0x4800_0000, 1).unwrap() as u8,
        bus.mem.read(0x4800_0001, 1).unwrap() as u8,
        bus.mem.read(0x4800_0002, 1).unwrap() as u8,
        bus.mem.read(0x4800_0003, 1).unwrap() as u8,
    ]);
    assert_eq!(magic, 0xd00dfeed);
}
