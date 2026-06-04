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
