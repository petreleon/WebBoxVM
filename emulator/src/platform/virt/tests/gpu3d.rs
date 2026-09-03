use super::*;

const DESC: u64 = RAM_BASE + 0x11000;
const AVAIL: u64 = RAM_BASE + 0x12000;
const USED: u64 = RAM_BASE + 0x13000;
const REQUEST: u64 = RAM_BASE + 0x14000;
const RESPONSE: u64 = RAM_BASE + 0x15000;

#[test]
fn browser_ack_finishes_deferred_submit_and_raises_external_irq() {
    let mut bus = SystemBus::new();
    configure_queue(&mut bus);
    queue_command(&mut bus, &context_create(), 1);
    bus.write(VIRTIO_GPU_BASE + 0x050, 4, 0);
    assert_eq!(bus.mem.read(USED + 2, 2), Some(1));
    bus.write(VIRTIO_GPU_BASE + 0x064, 4, 1);
    bus.clear_irq_pending(VIRTIO_GPU_IRQ_ID);
    bus.finish_cpu_instruction();

    queue_command(&mut bus, &submit_command(), 2);
    bus.begin_cpu_instruction();
    bus.write(VIRTIO_GPU_BASE + 0x050, 4, 0);
    assert_eq!(bus.mem.read(USED + 2, 2), Some(1));
    assert!(!bus.dma_write_during_instruction());
    assert!(!bus.gic.is_pending(VIRTIO_GPU_IRQ_ID));
    let packet = bus.virtio_gpu.take_3d_update();
    let sequence = read_u32(&packet, 12);
    assert_ne!(sequence, 99);

    assert!(bus.complete_gpu_3d(sequence, true));
    assert_eq!(bus.mem.read(USED + 2, 2), Some(2));
    assert_eq!(bus.mem.read(RESPONSE, 4), Some(0x1100));
    assert!(bus.take_external_dma_write());
    assert!(bus.gic.is_pending(VIRTIO_GPU_IRQ_ID));
}

fn configure_queue(bus: &mut SystemBus) {
    bus.write(VIRTIO_GPU_BASE + 0x038, 4, 8);
    set_queue_addr(bus, 0x080, DESC);
    set_queue_addr(bus, 0x090, AVAIL);
    set_queue_addr(bus, 0x0a0, USED);
    bus.write(VIRTIO_GPU_BASE + 0x044, 4, 1);
}

fn queue_command(bus: &mut SystemBus, command: &[u8], avail_index: u16) {
    bus.mem.write_bytes(REQUEST, command).unwrap();
    bus.mem.write_bytes(RESPONSE, &[0; 24]).unwrap();
    write_desc(bus, DESC, REQUEST, command.len() as u32, 1, 1);
    write_desc(bus, DESC + 16, RESPONSE, 24, 2, 0);
    let slot = u64::from((avail_index - 1) % 8);
    bus.mem.write(AVAIL + 4 + slot * 2, 2, 0).unwrap();
    bus.mem.write(AVAIL + 2, 2, u64::from(avail_index)).unwrap();
}

fn context_create() -> Vec<u8> {
    let mut command = header(0x0200);
    push_u32(&mut command, 4);
    push_u32(&mut command, 7);
    command.extend_from_slice(b"test");
    command.resize(96, 0);
    command
}

fn submit_command() -> Vec<u8> {
    let packet = packet();
    let mut command = header(0x0207);
    push_u32(&mut command, packet.len() as u32);
    push_u32(&mut command, 0);
    command.extend_from_slice(&packet);
    command
}

fn packet() -> Vec<u8> {
    let mut packet = b"WBG3".to_vec();
    for value in [1, 1, 99, 640, 480, 3, 3] {
        push_u32(&mut packet, value);
    }
    for value in [0.0f32, 0.0, 0.0, 1.0] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    for index in 0..16 {
        let value = if index % 5 == 0 { 1.0f32 } else { 0.0 };
        packet.extend_from_slice(&value.to_le_bytes());
    }
    for _ in 0..21 {
        packet.extend_from_slice(&0.25f32.to_le_bytes());
    }
    for index in [0u16, 1, 2] {
        packet.extend_from_slice(&index.to_le_bytes());
    }
    packet
}

fn header(command: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [command, 1] {
        push_u32(&mut bytes, value);
    }
    bytes.extend_from_slice(&0x1234_5678_9abc_def0u64.to_le_bytes());
    for value in [7, 0] {
        push_u32(&mut bytes, value);
    }
    bytes
}

fn set_queue_addr(bus: &mut SystemBus, offset: u64, addr: u64) {
    bus.write(VIRTIO_GPU_BASE + offset, 4, addr as u32 as u64);
    bus.write(VIRTIO_GPU_BASE + offset + 4, 4, addr >> 32);
}

fn write_desc(bus: &mut SystemBus, base: u64, addr: u64, len: u32, flags: u16, next: u16) {
    bus.mem.write(base, 8, addr).unwrap();
    bus.mem.write(base + 8, 4, u64::from(len)).unwrap();
    bus.mem.write(base + 12, 2, u64::from(flags)).unwrap();
    bus.mem.write(base + 14, 2, u64::from(next)).unwrap();
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
