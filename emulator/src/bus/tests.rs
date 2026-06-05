use super::*;

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
