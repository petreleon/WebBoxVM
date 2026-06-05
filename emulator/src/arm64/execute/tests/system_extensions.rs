use super::*;

#[test]
fn disabled_system_extensions_advance_without_mutation() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = RAM_BASE;
    cpu.regs.set_x(16, 1);
    cpu.regs.set_x(4, 0x4444);
    cpu.regs.set_x(5, 0x5555);

    execute(&mut cpu, &mut bus, decode(0xD503_251F).unwrap()).unwrap(); // chkfeat x16
    execute(&mut cpu, &mut bus, decode(0xD50B_7744).unwrap()).unwrap(); // gcsss1 x4
    execute(&mut cpu, &mut bus, decode(0xD52B_7725).unwrap()).unwrap(); // gcspopm x5
    execute(&mut cpu, &mut bus, decode(0xD503_467F).unwrap()).unwrap(); // smstop

    assert_eq!(cpu.regs.x(16), 1);
    assert_eq!(cpu.regs.x(4), 0x4444);
    assert_eq!(cpu.regs.x(5), 0x5555);
    assert_eq!(cpu.regs.pc, RAM_BASE + 16);
}

#[test]
fn mte_dc_tag_cache_ops_use_tagless_data_behavior() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, RAM_BASE + 13);
    bus.mem.write(RAM_BASE, 8, 0x1111_2222_3333_4444);
    bus.mem.write(RAM_BASE + 8, 8, 0x5555_6666_7777_8888);

    execute(&mut cpu, &mut bus, decode(0xD50B_7462).unwrap()).unwrap(); // dc gva, x2
    assert_eq!(bus.mem.read(RAM_BASE, 8), Some(0x1111_2222_3333_4444));
    assert_eq!(bus.mem.read(RAM_BASE + 8, 8), Some(0x5555_6666_7777_8888));

    execute(&mut cpu, &mut bus, decode(0xD50B_7482).unwrap()).unwrap(); // dc gzva, x2
    assert_eq!(bus.mem.read(RAM_BASE, 8), Some(0));
    assert_eq!(bus.mem.read(RAM_BASE + 8, 8), Some(0));
}

#[test]
fn sysl_writes_zero_result() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(3, 0xFFFF);

    execute(&mut cpu, &mut bus, decode(0xD528_7423).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(3), 0);
}

#[test]
fn flag_system_instructions_update_nzcv() {
    let (mut cpu, mut bus) = setup();
    cpu.pstate.set_nzcv(false, true, true, false);

    execute(&mut cpu, &mut bus, decode(0xD500_401F).unwrap()).unwrap(); // cfinv
    assert!(!cpu.pstate.c());

    cpu.regs.set_x(1, 0b1110);
    execute(&mut cpu, &mut bus, decode(0xBA00_042F).unwrap()).unwrap(); // rmif x1, #0, #15
    assert!(cpu.pstate.n() && cpu.pstate.z() && cpu.pstate.c());
    assert!(!cpu.pstate.v());

    cpu.regs.set_w(1, 0x80);
    execute(&mut cpu, &mut bus, decode(0x3A00_082D).unwrap()).unwrap(); // setf8 w1
    assert!(cpu.pstate.n() && cpu.pstate.v());
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.c());

    cpu.regs.set_w(1, 0);
    execute(&mut cpu, &mut bus, decode(0x3A00_482D).unwrap()).unwrap(); // setf16 w1
    assert!(cpu.pstate.z());
    assert!(!cpu.pstate.n() && !cpu.pstate.v());
    assert!(cpu.pstate.c());
}
