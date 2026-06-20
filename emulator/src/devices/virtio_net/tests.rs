use super::*;
use crate::constants::{VIRTIO_NET_BASE, VIRTIO_NET_IRQ_ID};
use crate::platform::virt::SystemBus;

const DESC: u64 = 0x4000_1000;
const AVAIL: u64 = 0x4000_2000;
const USED: u64 = 0x4000_3000;
const BUF: u64 = 0x4000_4000;

#[test]
fn mmio_identity_exposes_virtio_net() {
    let device = VirtioNet::new();

    assert_eq!(device.read(0x000, 4), Some(VIRTIO_MMIO_MAGIC));
    assert_eq!(device.read(0x004, 4), Some(VIRTIO_MMIO_VERSION_2));
    assert_eq!(device.read(0x008, 4), Some(VIRTIO_DEVICE_ID_NET));
    assert_eq!(device.read(0x100, 1), Some(0x02));
    assert_eq!(device.read(0x106, 2), Some(1));
}

#[test]
fn selected_queues_have_independent_registers() {
    let mut device = VirtioNet::new();
    let mut mem = PhysicalMemory::new();

    device.write(&mut mem, 0x030, QUEUE_RX as u64, 4);
    device.write(&mut mem, 0x038, 8, 4);
    device.write(&mut mem, 0x080, DESC, 4);
    device.write(&mut mem, 0x044, 1, 4);
    device.write(&mut mem, 0x030, QUEUE_TX as u64, 4);

    assert_eq!(device.read(0x034, 4), Some(QUEUE_NUM_MAX as u64));
    assert_eq!(device.read(0x044, 4), Some(0));
    assert_eq!(device.read(0x080, 4), Some(0));
}

#[test]
fn transmit_queue_exports_ethernet_frame() {
    let mut bus = SystemBus::new();
    configure_queue(&mut bus, QUEUE_TX);
    assert_eq!(VIRTIO_NET_HDR_LEN, 12);
    write_desc(&mut bus, 0, BUF, (VIRTIO_NET_HDR_LEN + 4) as u32, 0, 0);
    bus.mem.write_bytes(BUF, &[0; VIRTIO_NET_HDR_LEN]).unwrap();
    bus.mem
        .write_bytes(BUF + VIRTIO_NET_HDR_LEN as u64, &[1, 2, 3, 4])
        .unwrap();
    publish_avail(&mut bus, 0);

    bus.write(VIRTIO_NET_BASE + 0x050, 4, QUEUE_TX as u64);

    assert_eq!(bus.virtio_net.pop_tx_frame(), Some(vec![1, 2, 3, 4]));
    assert!(bus.gic.is_pending(VIRTIO_NET_IRQ_ID));
    assert_eq!(bus.mem.read(USED + 2, 2), Some(1));
}

#[test]
fn receive_queue_injects_frame_into_guest_buffer() {
    let mut bus = SystemBus::new();
    configure_queue(&mut bus, QUEUE_RX);
    write_desc(&mut bus, 0, BUF, 64, VIRTQ_DESC_F_WRITE, 0);
    publish_avail(&mut bus, 0);

    bus.inject_network_frame(&[0xde, 0xad, 0xbe, 0xef]);

    let mut packet = [0u8; VIRTIO_NET_HDR_LEN + 4];
    bus.mem.read_bytes(BUF, &mut packet).unwrap();
    assert_eq!(&packet[..VIRTIO_NET_HDR_LEN], &[0; VIRTIO_NET_HDR_LEN]);
    assert_eq!(&packet[VIRTIO_NET_HDR_LEN..], &[0xde, 0xad, 0xbe, 0xef]);
    assert!(bus.gic.is_pending(VIRTIO_NET_IRQ_ID));
    assert_eq!(
        bus.mem.read(USED + 4 + 4, 4),
        Some((VIRTIO_NET_HDR_LEN + 4) as u64),
    );
}

fn configure_queue(bus: &mut SystemBus, queue: usize) {
    bus.write(VIRTIO_NET_BASE + 0x030, 4, queue as u64);
    bus.write(VIRTIO_NET_BASE + 0x038, 4, 8);
    bus.write(VIRTIO_NET_BASE + 0x080, 4, DESC);
    bus.write(VIRTIO_NET_BASE + 0x090, 4, AVAIL);
    bus.write(VIRTIO_NET_BASE + 0x0a0, 4, USED);
    bus.write(VIRTIO_NET_BASE + 0x044, 4, 1);
}

fn write_desc(bus: &mut SystemBus, index: u16, addr: u64, len: u32, flags: u16, next: u16) {
    let base = DESC + index as u64 * 16;
    bus.mem.write(base, 8, addr).unwrap();
    bus.mem.write(base + 8, 4, len as u64).unwrap();
    bus.mem.write(base + 12, 2, flags as u64).unwrap();
    bus.mem.write(base + 14, 2, next as u64).unwrap();
}

fn publish_avail(bus: &mut SystemBus, head: u16) {
    bus.mem.write(AVAIL + 4, 2, head as u64).unwrap();
    bus.mem.write(AVAIL + 2, 2, 1).unwrap();
}
