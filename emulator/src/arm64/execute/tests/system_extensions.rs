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
