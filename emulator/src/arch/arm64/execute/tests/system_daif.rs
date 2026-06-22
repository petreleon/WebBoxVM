use super::*;

#[test]
fn daifset_three_masks_irq() {
    let (mut cpu, mut bus) = setup();
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xd503_43df).unwrap()).unwrap();

    assert!(cpu.pstate.irq_masked());
}

#[test]
fn daifclr_three_unmasks_irq() {
    let (mut cpu, mut bus) = setup();
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(true);

    execute(&mut cpu, &mut bus, decode(0xd503_43ff).unwrap()).unwrap();

    assert!(!cpu.pstate.irq_masked());
}
