use super::*;

#[test]
fn sme_za_array_memory_traps_until_za_state_is_modeled() {
    let (mut cpu, mut bus) = setup();
    cpu.pstate = cpu.pstate.with_el(0);
    cpu.regs.pc = RAM_BASE;
    cpu.sys.vbar_el1 = RAM_BASE + 0x1000;
    cpu.regs.set_x(16, RAM_BASE + 0x2000);

    execute(&mut cpu, &mut bus, decode(0xE100_620F).unwrap()).unwrap();
    assert_eq!(cpu.sys.elr_el1, RAM_BASE);

    cpu.pstate = cpu.pstate.with_el(0);
    cpu.regs.pc = RAM_BASE + 4;
    execute(&mut cpu, &mut bus, decode(0xE120_620F).unwrap()).unwrap();

    assert_eq!(cpu.sys.elr_el1, RAM_BASE + 4);
    assert_eq!(cpu.sys.esr_el1, (ESR_EC_UNKNOWN << 26) | ESR_IL);
    assert_eq!(cpu.regs.pc, RAM_BASE + 0x1000 + VBAR_SYNC_LOWER_EL_AARCH64);
    assert_eq!(bus.mem.read(RAM_BASE + 0x2000, 8), Some(0));
}
