use super::*;

#[test]
fn mops_forward_copy_triplet_copies_bytes_and_consumes_size() {
    let (mut cpu, mut bus) = setup();
    for i in 0..8 {
        bus.mem.write(RAM_BASE + 0x100 + i, 1, 0xA0 + i);
    }
    cpu.regs.set_x(1, RAM_BASE + 0x100);
    cpu.regs.set_x(2, 8);
    cpu.regs.set_x(3, RAM_BASE + 0x200);

    for raw in [0x1901_0443, 0x1941_0443, 0x1981_0443] {
        execute(&mut cpu, &mut bus, decode(raw).unwrap()).unwrap();
    }

    for i in 0..8 {
        assert_eq!(bus.mem.read(RAM_BASE + 0x200 + i, 1), Some(0xA0 + i));
    }
    assert_eq!(cpu.regs.x(1), RAM_BASE + 0x108);
    assert_eq!(cpu.regs.x(2), 0);
    assert_eq!(cpu.regs.x(3), RAM_BASE + 0x208);
    assert!(cpu.pstate.c());
}

#[test]
fn mops_overlap_copy_uses_backward_direction() {
    let (mut cpu, mut bus) = setup();
    for i in 0..8 {
        bus.mem.write(RAM_BASE + i, 1, i);
    }
    cpu.regs.set_x(1, RAM_BASE);
    cpu.regs.set_x(2, 8);
    cpu.regs.set_x(3, RAM_BASE + 2);

    execute(&mut cpu, &mut bus, decode(0x1D01_0443).unwrap()).unwrap();

    let copied = [0, 1, 2, 3, 4, 5, 6, 7];
    for (i, expected) in copied.into_iter().enumerate() {
        assert_eq!(bus.mem.read(RAM_BASE + 2 + i as u64, 1), Some(expected));
    }
    assert!(cpu.pstate.n());
    assert!(cpu.pstate.c());
}

#[test]
fn mops_set_triplet_writes_low_source_byte() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0xABCD);
    cpu.regs.set_x(2, 6);
    cpu.regs.set_x(3, RAM_BASE + 0x300);

    for raw in [0x19C1_0443, 0x19C1_4443, 0x19C1_8443] {
        execute(&mut cpu, &mut bus, decode(raw).unwrap()).unwrap();
    }

    for i in 0..6 {
        assert_eq!(bus.mem.read(RAM_BASE + 0x300 + i, 1), Some(0xCD));
    }
    assert_eq!(cpu.regs.x(2), 0);
    assert_eq!(cpu.regs.x(3), RAM_BASE + 0x306);
    assert!(cpu.pstate.c());
}
