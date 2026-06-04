use super::*;

#[test]
fn sve_scalar_vector_length_forms_use_configured_lengths() {
    let (mut cpu, mut bus) = setup();

    execute(&mut cpu, &mut bus, decode(0x0420_E3E0).unwrap()).unwrap(); // cntb x0
    assert_eq!(cpu.regs.x(0), 16);

    execute(&mut cpu, &mut bus, decode(0x04E0_E3E0).unwrap()).unwrap(); // cntd x0
    assert_eq!(cpu.regs.x(0), 2);

    execute(&mut cpu, &mut bus, decode(0x04E1_E3E0).unwrap()).unwrap(); // cntd x0, all, mul #2
    assert_eq!(cpu.regs.x(0), 4);

    execute(&mut cpu, &mut bus, decode(0x04E0_E020).unwrap()).unwrap(); // cntd x0, vl1
    assert_eq!(cpu.regs.x(0), 1);

    execute(&mut cpu, &mut bus, decode(0x04E0_E080).unwrap()).unwrap(); // cntd x0, vl4
    assert_eq!(cpu.regs.x(0), 0);

    cpu.regs.sp = 0x1000;
    execute(&mut cpu, &mut bus, decode(0x043F_57FF).unwrap()).unwrap(); // addvl sp, sp, #-1
    assert_eq!(cpu.regs.sp, 0x0ff0);

    cpu.sme_svl_bytes = 32;
    cpu.regs.set_x(16, 0x2000);
    execute(&mut cpu, &mut bus, decode(0x0430_5A10).unwrap()).unwrap(); // addsvl x16, x16, #16
    assert_eq!(cpu.regs.x(16), 0x2200);
}
