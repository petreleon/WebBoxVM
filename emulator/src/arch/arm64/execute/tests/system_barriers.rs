use super::*;

#[test]
fn clrex_clears_exclusive_monitor() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = RAM_BASE;
    cpu.reserve_exclusive(RAM_BASE + 0x100, 8);

    let instr = decode(0xD503_305F).unwrap();
    assert_eq!(instr.op, Opcode::Clrex);

    execute(&mut cpu, &mut bus, instr).unwrap();

    assert!(cpu.exclusive.is_none());
    assert_eq!(cpu.regs.pc, RAM_BASE + 4);
}

#[test]
fn barriers_advance_without_mutation() {
    let cases = [
        (0xD503_3BBF, Opcode::Dmb),
        (0xD503_3B9F, Opcode::Dsb),
        (0xD503_323F, Opcode::Dsb),
        (0xD503_3FDF, Opcode::Isb),
    ];

    for (raw, expected) in cases {
        let (mut cpu, mut bus) = setup();
        cpu.regs.pc = RAM_BASE;
        cpu.regs.set_x(2, RAM_BASE + 0x1000);

        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        execute(&mut cpu, &mut bus, instr).unwrap();

        assert_eq!(cpu.regs.pc, RAM_BASE + 4, "raw=0x{raw:08x}");
        assert_eq!(cpu.regs.x(2), RAM_BASE + 0x1000, "raw=0x{raw:08x}");
    }
}
