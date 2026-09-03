use super::*;

#[test]
fn virtio_gpu_has_device_id_and_own_mmio_window() {
    let mut bus = SystemBus::new();
    assert_eq!(bus.read(VIRTIO_GPU_BASE, 4), Some(0x7472_6976));
    assert_eq!(bus.read(VIRTIO_GPU_BASE + 0x008, 4), Some(16));
    assert_eq!(bus.read(VIRTIO_GPU_BASE + 0x108, 4), Some(1));
}

#[test]
fn virtio_gpu_reports_no_shared_memory_region() {
    let mut bus = SystemBus::new();
    bus.write(VIRTIO_GPU_BASE + 0x0ac, 4, 0);
    assert_eq!(bus.read(VIRTIO_GPU_BASE + 0x0b0, 4), Some(u32::MAX as u64));
    assert_eq!(bus.read(VIRTIO_GPU_BASE + 0x0b4, 4), Some(u32::MAX as u64));
    assert_eq!(bus.read(VIRTIO_GPU_BASE + 0x0b8, 4), Some(0));
    assert_eq!(bus.read(VIRTIO_GPU_BASE + 0x0bc, 4), Some(0));
}

#[test]
fn gpu_window_is_protected_from_bulk_memory_writes() {
    let mut bus = SystemBus::new();
    assert_eq!(bus.write_bytes(VIRTIO_GPU_BASE, &[1, 2, 3, 4]), None);
    assert!(bus.overlaps_device_range(VIRTIO_GPU_BASE, 1));
    assert!(bus.overlaps_device_range(VIRTIO_GPU_END - 1, 1));
    assert!(!bus.overlaps_device_range(VIRTIO_GPU_END, 1));
}

#[test]
fn gpu_control_queue_completion_records_dma_and_raises_irq() {
    let mut bus = SystemBus::new();
    let desc = RAM_BASE + 0x1000;
    let avail = RAM_BASE + 0x2000;
    let used = RAM_BASE + 0x3000;
    let request = RAM_BASE + 0x4000;
    let response = RAM_BASE + 0x5000;
    bus.mem
        .write_bytes(request, &0x0100u32.to_le_bytes())
        .unwrap();
    write_desc(&mut bus, desc, request, 24, 1, 1);
    write_desc(&mut bus, desc + 16, response, 512, 2, 0);
    bus.mem.write(avail + 2, 2, 1).unwrap();
    bus.mem.write(avail + 4, 2, 0).unwrap();
    bus.write(VIRTIO_GPU_BASE + 0x038, 4, 8);
    set_queue_addr(&mut bus, 0x080, desc);
    set_queue_addr(&mut bus, 0x090, avail);
    set_queue_addr(&mut bus, 0x0a0, used);
    bus.write(VIRTIO_GPU_BASE + 0x044, 4, 1);

    bus.begin_cpu_instruction();
    bus.write(VIRTIO_GPU_BASE + 0x050, 4, 0);
    assert!(bus.dma_write_during_instruction());
    assert!(bus.gic.is_pending(VIRTIO_GPU_IRQ_ID));
    assert_eq!(bus.mem.read(response, 4), Some(0x1101));
    assert_eq!(bus.mem.read(used + 2, 2), Some(1));
}

fn set_queue_addr(bus: &mut SystemBus, offset: u64, addr: u64) {
    bus.write(VIRTIO_GPU_BASE + offset, 4, addr as u32 as u64);
    bus.write(VIRTIO_GPU_BASE + offset + 4, 4, addr >> 32);
}

fn write_desc(bus: &mut SystemBus, base: u64, addr: u64, len: u32, flags: u16, next: u16) {
    bus.mem.write(base, 8, addr).unwrap();
    bus.mem.write(base + 8, 4, len as u64).unwrap();
    bus.mem.write(base + 12, 2, flags as u64).unwrap();
    bus.mem.write(base + 14, 2, next as u64).unwrap();
}
