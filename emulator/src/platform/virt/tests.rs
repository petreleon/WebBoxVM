use super::*;
use crate::api::{AccessWidth, PhysAddr};

#[test]
fn device_mmio_windows_are_disjoint() {
    let windows = [
        ("gicd", GICD_BASE, GICD_BASE + GICD_SIZE),
        ("gicr", GICR_BASE, GICR_BASE + GICR_SIZE),
        ("uart", UART_BASE, UART_END),
        ("virtio_blk", VIRTIO_BLK_BASE, VIRTIO_BLK_END),
        ("virtio_disk", VIRTIO_DISK_BASE, VIRTIO_DISK_END),
    ];

    for (i, (left_name, left_base, left_end)) in windows.iter().enumerate() {
        for (right_name, right_base, right_end) in windows.iter().skip(i + 1) {
            assert!(
                left_end <= right_base || right_end <= left_base,
                "{left_name} overlaps {right_name}"
            );
        }
    }
}

#[test]
fn uart_priority_over_ram() {
    let mut bus = SystemBus::new();
    bus.write(UART_BASE, 1, b'A' as u64);
    assert_eq!(bus.uart.output_string(), "A");
}

#[test]
fn ram_read_write() {
    let mut bus = SystemBus::new();
    bus.write(RAM_BASE, 8, 0xDEADBEEF);
    assert_eq!(bus.read(RAM_BASE, 8), Some(0xDEADBEEF));
}

#[test]
fn typed_physical_access_preserves_address_and_width() {
    let mut bus = SystemBus::new();
    let addr = PhysAddr::new(RAM_BASE + 0x200);

    bus.write_phys(addr, AccessWidth::Word, 0xAABB_CCDD);

    assert_eq!(bus.read_phys(addr, AccessWidth::Byte), Some(0xDD));
    assert_eq!(bus.read_phys(addr, AccessWidth::Word), Some(0xAABB_CCDD));
}

#[test]
fn second_virtio_disk_has_own_mmio_window() {
    let mut bus = SystemBus::new();
    assert_eq!(bus.read(VIRTIO_BLK_BASE, 4), Some(0x7472_6976));
    assert_eq!(bus.read(VIRTIO_DISK_BASE, 4), Some(0x7472_6976));
}

#[test]
fn bulk_write_rejects_device_ranges() {
    let mut bus = SystemBus::new();
    assert_eq!(bus.write_bytes(RAM_BASE + 0x100, &[1, 2, 3, 4]), Some(()));
    assert_eq!(bus.mem.read(RAM_BASE + 0x100, 4), Some(0x0403_0201));
    assert_eq!(bus.write_bytes(UART_BASE, b"A"), None);
}

#[test]
fn device_overlap_detection_handles_edges_and_overflow() {
    let bus = SystemBus::new();

    assert!(!bus.overlaps_device_range(UART_BASE, 0));
    assert!(!bus.overlaps_device_range(VIRTIO_BLK_BASE - 1, 1));
    assert!(bus.overlaps_device_range(VIRTIO_BLK_BASE - 1, 2));
    assert!(bus.overlaps_device_range(UART_END - 1, 1));
    assert!(!bus.overlaps_device_range(UART_END, 1));
    assert!(bus.overlaps_device_range(u64::MAX - 1, 8));
}

#[test]
fn refresh_interrupts_reasserts_uart_rx_while_input_remains() {
    let mut bus = SystemBus::new();
    bus.write(UART_BASE + 0x38, 4, 0x50);
    bus.uart.feed_input("ab");

    bus.gic.clear_pending(PL011_UART_IRQ_ID);
    bus.refresh_interrupts();
    assert!(bus.gic.is_pending(PL011_UART_IRQ_ID));

    assert_eq!(bus.read(UART_BASE, 1), Some(b'a' as u64));
    bus.gic.clear_pending(PL011_UART_IRQ_ID);
    bus.refresh_interrupts();
    assert!(bus.gic.is_pending(PL011_UART_IRQ_ID));

    assert_eq!(bus.read(UART_BASE, 1), Some(b'b' as u64));
    bus.gic.clear_pending(PL011_UART_IRQ_ID);
    bus.refresh_interrupts();
    assert!(!bus.gic.is_pending(PL011_UART_IRQ_ID));
}
