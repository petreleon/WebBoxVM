use super::*;

const NOP: u64 = 0xd503_201f;
const WFI: u64 = 0xd503_207f;
const STR_X0_X1: u64 = 0xf900_0020;
const NET_DESC: u64 = RAM_BASE + 0x8000;
const NET_AVAIL: u64 = RAM_BASE + 0x9000;
const NET_USED: u64 = RAM_BASE + 0xa000;
const NET_BUF: u64 = RAM_BASE + 0xb000;

#[test]
fn guest_store_invalidates_overlapping_reservations_on_other_cpus() {
    let mut machine = Machine::new(2);
    let code = RAM_BASE + 0x1000;
    let data = RAM_BASE + 0x2000;
    machine.bus.mem.write(code, 4, STR_X0_X1);
    machine.cpus[0].regs.pc = code;
    machine.cpus[0].regs.set_x(0, 0xfeed_face);
    machine.cpus[0].regs.set_x(1, data);
    machine.cpus[1].reserve_exclusive(data, 8);

    assert_eq!(machine.run(1), 1);
    assert!(machine.cpus[1].exclusive.is_none());
    assert_eq!(machine.bus.mem.read(data, 8), Some(0xfeed_face));
}

#[test]
fn guest_store_preserves_non_overlapping_reservations() {
    let mut machine = Machine::new(2);
    let code = RAM_BASE + 0x3000;
    let data = RAM_BASE + 0x4000;
    machine.bus.mem.write(code, 4, STR_X0_X1);
    machine.cpus[0].regs.pc = code;
    machine.cpus[0].regs.set_x(1, data);
    machine.cpus[1].reserve_exclusive(data + 0x100, 8);

    machine.run(1);

    assert!(machine.cpus[1].exclusive_matches(data + 0x100, 8));
}

#[test]
fn guest_store_invalidates_a_reservation_in_the_same_erg() {
    let mut machine = Machine::new(2);
    let code = RAM_BASE + 0x6000;
    let data = RAM_BASE + 0x7000;
    machine.bus.mem.write(code, 4, STR_X0_X1);
    machine.cpus[0].regs.pc = code;
    machine.cpus[0].regs.set_x(1, data);
    machine.cpus[1].reserve_exclusive(data + 0x20, 8);

    machine.run(1);

    assert!(machine.cpus[1].exclusive.is_none());
}

#[test]
fn wfi_parks_and_a_private_interrupt_wakes_the_cpu() {
    let mut machine = Machine::new(1);
    let code = RAM_BASE + 0x5000;
    machine.bus.mem.write(code, 4, WFI);
    machine.bus.mem.write(code + 4, 4, NOP);
    machine.cpus[0].regs.pc = code;

    assert_eq!(machine.run(1), 1);
    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::WaitingForInterrupt);
    assert_eq!(machine.run(1), 0);

    machine.bus.gic.enable_interrupt_for_cpu(0, 7);
    machine.bus.gic.set_pending_for_cpu(0, 7);
    assert_eq!(machine.run(1), 1);
    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::Runnable);
}

#[test]
fn virtio_dma_during_a_guest_instruction_invalidates_reservations() {
    let mut machine = Machine::new(2);
    configure_net_queue(&mut machine, 1);
    write_net_desc(&mut machine, NET_BUF, 16, 0);
    machine.bus.mem.write_bytes(NET_BUF, &[0; 16]).unwrap();
    publish_net_desc(&mut machine);

    let code = RAM_BASE + 0xc000;
    machine.bus.mem.write(code, 4, STR_X0_X1);
    machine.cpus[0].regs.pc = code;
    machine.cpus[0].regs.set_x(0, 1);
    machine.cpus[0].regs.set_x(1, VIRTIO_NET_BASE + 0x050);
    machine.cpus[1].reserve_exclusive(NET_USED + 2, 2);

    assert_eq!(machine.run(1), 1);
    assert!(machine.cpus[1].exclusive.is_none());
}

#[test]
fn external_virtio_dma_is_applied_before_the_next_cpu_instruction() {
    let mut machine = Machine::new(2);
    configure_net_queue(&mut machine, 0);
    write_net_desc(&mut machine, NET_BUF, 64, 2);
    publish_net_desc(&mut machine);

    let code = RAM_BASE + 0xd000;
    machine.bus.mem.write(code, 4, NOP);
    machine.cpus[0].regs.pc = code;
    machine.cpus[1].reserve_exclusive(NET_BUF, 8);

    machine.bus.inject_network_frame(&[1, 2, 3, 4]);
    assert!(machine.cpus[1].exclusive_matches(NET_BUF, 8));

    assert_eq!(machine.run(1), 1);
    assert!(machine.cpus[1].exclusive.is_none());
}

#[test]
fn machine_network_injection_invalidates_reservations_immediately() {
    let mut machine = Machine::new(2);
    configure_net_queue(&mut machine, 0);
    write_net_desc(&mut machine, NET_BUF, 64, 2);
    publish_net_desc(&mut machine);
    machine.cpus[1].reserve_exclusive(NET_BUF, 8);

    machine.inject_network_frame(&[1, 2, 3, 4]);

    assert!(machine.cpus[1].exclusive.is_none());
}

fn configure_net_queue(machine: &mut Machine, queue: u64) {
    machine.bus.write(VIRTIO_NET_BASE + 0x030, 4, queue);
    machine.bus.write(VIRTIO_NET_BASE + 0x038, 4, 8);
    machine.bus.write(VIRTIO_NET_BASE + 0x080, 4, NET_DESC);
    machine.bus.write(VIRTIO_NET_BASE + 0x090, 4, NET_AVAIL);
    machine.bus.write(VIRTIO_NET_BASE + 0x0a0, 4, NET_USED);
    machine.bus.write(VIRTIO_NET_BASE + 0x044, 4, 1);
}

fn write_net_desc(machine: &mut Machine, addr: u64, len: u32, flags: u16) {
    machine.bus.mem.write(NET_DESC, 8, addr).unwrap();
    machine.bus.mem.write(NET_DESC + 8, 4, len as u64).unwrap();
    machine
        .bus
        .mem
        .write(NET_DESC + 12, 2, flags as u64)
        .unwrap();
    machine.bus.mem.write(NET_DESC + 14, 2, 0).unwrap();
}

fn publish_net_desc(machine: &mut Machine) {
    machine.bus.mem.write(NET_AVAIL + 4, 2, 0).unwrap();
    machine.bus.mem.write(NET_AVAIL + 2, 2, 1).unwrap();
}
