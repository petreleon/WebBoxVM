use super::super::*;

#[test]
fn config_read_rejects_out_of_range_sizes() {
    let device = VirtioBlk::writable_sparse(SECTOR_SIZE as u64, b"disk\0");

    assert_eq!(device.read(0x106, 4), None);
    assert_eq!(device.read(0x107, 2), None);
    assert_eq!(device.read(0x100, 8), Some(1));
}
