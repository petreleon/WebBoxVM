use super::*;

#[test]
fn addsub_extended_register_forms_apply_extension_and_shift() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_x(1, 100);
    cpu.regs.set_w(2, 3);
    execute(&mut cpu, &mut bus, decode(0x8B22_4820).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 112);

    cpu.regs.set_x(4, 100);
    cpu.regs.set_x(5, u64::MAX);
    execute(&mut cpu, &mut bus, decode(0xCB25_E483).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(3), 102);

    cpu.regs.set_w(7, 0xffff_fff0);
    cpu.regs.set_w(8, 2);
    execute(&mut cpu, &mut bus, decode(0x2B28_0CE6).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(6), 0);
    assert!(cpu.pstate.z());
    assert!(cpu.pstate.c());

    cpu.regs.set_x(10, 0);
    cpu.regs.set_w(11, 0xffff);
    execute(&mut cpu, &mut bus, decode(0xEB2B_A949).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(9), 4);

    cpu.regs.set_x(12, 0x400);
    cpu.regs.set_w(13, 0x100);
    execute(&mut cpu, &mut bus, decode(0xEB2D_499F).unwrap()).unwrap();
    assert!(cpu.pstate.z());
}
