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
        ("virtio_net", VIRTIO_NET_BASE, VIRTIO_NET_END),
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
fn mmio_writes_do_not_shadow_sparse_memory() {
    let mut bus = SystemBus::new();

    bus.write(UART_BASE, 1, b'A' as u64);
    bus.write(GICD_BASE, 4, 1);

    assert_eq!(bus.uart.output_string(), "A");
    assert_eq!(bus.mem.allocated_pages(), 0);
    assert_eq!(bus.mem.page_generation(UART_BASE), Some(0));
    assert_eq!(bus.mem.page_generation(GICD_BASE), Some(0));
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
fn virtio_net_has_own_mmio_window() {
    let mut bus = SystemBus::new();
    assert_eq!(bus.read(VIRTIO_NET_BASE, 4), Some(0x7472_6976));
    assert_eq!(bus.read(VIRTIO_NET_BASE + 0x008, 4), Some(1));
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
    bus.feed_uart_input("ab");

    bus.clear_irq_pending(PL011_UART_IRQ_ID);
    bus.refresh_interrupts();
    assert!(bus.gic.is_pending(PL011_UART_IRQ_ID));

    assert_eq!(bus.read(UART_BASE, 1), Some(b'a' as u64));
    bus.clear_irq_pending(PL011_UART_IRQ_ID);
    bus.refresh_interrupts();
    assert!(bus.gic.is_pending(PL011_UART_IRQ_ID));

    assert_eq!(bus.read(UART_BASE, 1), Some(b'b' as u64));
    bus.clear_irq_pending(PL011_UART_IRQ_ID);
    bus.refresh_interrupts();
    assert!(!bus.gic.is_pending(PL011_UART_IRQ_ID));
}

#[test]
fn gicd_clear_pending_marks_uart_refresh_needed() {
    let mut bus = SystemBus::new();
    let clear_pending = GICD_BASE + 0x280 + ((PL011_UART_IRQ_ID / 32) as u64) * 4;
    let bit = 1u64 << (PL011_UART_IRQ_ID % 32);

    bus.write(UART_BASE + UART_IMSC_OFFSET, 4, 0x50);
    bus.feed_uart_input("x");
    bus.write(clear_pending, 4, bit);

    assert!(!bus.gic.is_pending(PL011_UART_IRQ_ID));
    bus.refresh_interrupts();
    assert!(bus.gic.is_pending(PL011_UART_IRQ_ID));
}

#[test]
fn uart_imsc_write_marks_rx_refresh_needed() {
    let mut bus = SystemBus::new();
    bus.uart.feed_input("x");

    bus.write(UART_BASE + UART_IMSC_OFFSET, 4, 0x50);

    assert!(!bus.gic.is_pending(PL011_UART_IRQ_ID));
    bus.refresh_interrupts();
    assert!(bus.gic.is_pending(PL011_UART_IRQ_ID));
}
