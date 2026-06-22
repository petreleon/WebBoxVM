use super::*;

#[test]
fn u64_roundtrip() {
    let mut m = PhysicalMemory::new();
    assert!(m.write(RAM_BASE, 8, 0xCAFE0000_DEADBEEF).is_some());
    assert_eq!(m.read(RAM_BASE, 8), Some(0xCAFE0000_DEADBEEF));
}

#[test]
fn kernel_region_roundtrip() {
    let mut m = PhysicalMemory::new();
    assert!(m.write(0x1_0000, 8, 0x1234_5678_9ABC_DEFF).is_some());
    assert_eq!(m.read(0x1_0000, 8), Some(0x1234_5678_9ABC_DEFF));
}

#[test]
fn u8_roundtrip() {
    let mut m = PhysicalMemory::new();
    assert!(m.write(0x4000_0100, 1, 0x42).is_some());
    assert_eq!(m.read(0x4000_0100, 1), Some(0x42));
}

#[test]
fn unmapped_fails() {
    let m = PhysicalMemory::new();
    assert_eq!(m.read(0x0000_0000, 4), Some(0));
}

#[test]
fn new_memory_does_not_allocate_guest_pages() {
    let m = PhysicalMemory::new();
    assert_eq!(m.allocated_pages(), 0);
    assert_eq!(m.read(RAM_BASE + 0x1000, 8), Some(0));
    assert_eq!(m.allocated_pages(), 0);
}

#[test]
fn bulk_access_crosses_sparse_pages() {
    let mut m = PhysicalMemory::new();
    let addr = RAM_BASE + PAGE_SIZE - 2;
    let bytes = [1, 2, 3, 4, 5];
    let mut out = [0u8; 5];

    m.write_bytes(addr, &bytes).unwrap();
    m.read_bytes(addr, &mut out).unwrap();

    assert_eq!(out, bytes);
    assert_eq!(m.read(addr, 4), Some(0x0403_0201));
    assert_eq!(m.read_u32(addr), Some(0x0403_0201));
    assert_eq!(m.allocated_pages(), 2);
}

#[test]
fn fixed_read_crosses_sparse_pages() {
    let mut m = PhysicalMemory::new();
    let addr = RAM_BASE + PAGE_SIZE - 4;
    let bytes = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];

    m.write_bytes(addr, &bytes).unwrap();

    assert_eq!(m.read_u64(addr), Some(0x8877_6655_4433_2211));
}

#[test]
fn scalar_write_crosses_sparse_pages() {
    let mut m = PhysicalMemory::new();
    let addr = RAM_BASE + PAGE_SIZE - 2;

    m.write(addr, 4, 0xAABB_CCDD).unwrap();

    assert_eq!(m.read(addr, 4), Some(0xAABB_CCDD));
    assert_eq!(m.allocated_pages(), 2);
    assert_eq!(m.page_generation(addr), Some(1));
    assert_eq!(m.page_generation(addr + 2), Some(1));
}

#[test]
fn page_generation_tracks_writes() {
    let mut m = PhysicalMemory::new();
    let addr = RAM_BASE + 0x100;

    assert_eq!(m.page_generation(addr), Some(0));
    m.write(addr, 4, 0x1234).unwrap();
    assert_eq!(m.page_generation(addr), Some(1));
    m.write(addr, 4, 0x1234).unwrap();
    assert_eq!(m.page_generation(addr), Some(2));
}

#[test]
fn page_generation_uses_single_address_region_bounds() {
    let m = PhysicalMemory::new();

    assert_eq!(m.page_generation(LOW_REGION_BASE), Some(0));
    assert_eq!(m.page_generation(LOW_REGION_END - 1), Some(0));
    assert_eq!(m.page_generation(RAM_BASE), Some(0));
    assert_eq!(m.page_generation(RAM_END - 1), Some(0));
    assert_eq!(m.page_generation(EFI_REGION_BASE), Some(0));
    assert_eq!(m.page_generation(EFI_REGION_END - 1), Some(0));
    assert_eq!(m.page_generation(EFI_REGION_END), None);
    assert_eq!(m.page_generation(u64::MAX), None);
}

#[test]
fn range_must_stay_inside_one_region() {
    let mut m = PhysicalMemory::new();
    let mut out = [0u8; 3];

    assert_eq!(m.write_bytes(LOW_REGION_END - 2, &[1, 2, 3]), None);
    assert_eq!(m.read_bytes(LOW_REGION_END - 2, &mut out), None);
    assert_eq!(m.read(EFI_REGION_END, 1), None);
}
