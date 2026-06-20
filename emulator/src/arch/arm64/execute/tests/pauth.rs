use super::*;

#[test]
fn pauth_aliases_advance_without_mutation() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = RAM_BASE;
    cpu.regs.set_x(0, 0xF00D);
    cpu.regs.set_x(30, 0xCAFE);

    for raw in [
        0xD503_233F,
        0xD503_23BF,
        0xD503_20FF,
        0xDAC1_0000,
        0xDAC1_0400,
        0xDAC1_0800,
        0xDAC1_0C00,
        0xDAC1_1000,
        0xDAC1_1400,
        0xDAC1_1800,
        0xDAC1_1C00,
        0xDAC1_23E0,
        0xDAC1_27E0,
        0xDAC1_2BE0,
        0xDAC1_2FE0,
        0xDAC1_33E0,
        0xDAC1_37E0,
        0xDAC1_3BE0,
        0xDAC1_3FE0,
        0xDAC1_43E0,
        0xDAC1_47E0,
    ] {
        execute(&mut cpu, &mut bus, decode(raw).unwrap()).unwrap();
    }

    assert_eq!(cpu.regs.x(0), 0xF00D);
    assert_eq!(cpu.regs.x(30), 0xCAFE);
    assert_eq!(cpu.regs.pc, RAM_BASE + 84);
}
