use super::*;

#[test]
fn sve_whilelo_builds_unsigned_predicate_and_flags() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;
    cpu.regs.set_x(7, 0x1_0000_0003);
    cpu.regs.set_x(2, 7);
    cpu.pstate.set_nzcv(false, false, false, true);

    execute(&mut cpu, &mut bus, decode(0x2522_0CE1).unwrap()).unwrap();

    for bit in 0..4 {
        assert!(pred_bit(&cpu, 1, bit));
    }
    assert!(!pred_bit(&cpu, 1, 4));
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.c());
    assert!(!cpu.pstate.v());
    assert_eq!(cpu.regs.x(7), 0x1_0000_0003);

    cpu.sve_pred[0] = [u64::MAX; 4];
    cpu.regs.set_x(2, 0);
    execute(&mut cpu, &mut bus, decode(0x2562_0FE0).unwrap()).unwrap();
    assert_eq!(cpu.sve_pred[0], [0; 4]);
    assert!(!cpu.pstate.n());
    assert!(cpu.pstate.z());
    assert!(cpu.pstate.c());
}

#[test]
fn sve_whilelo_uses_x_width_and_element_stride() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;
    cpu.regs.set_x(5, u64::MAX - 1);
    cpu.regs.set_x(9, u64::MAX);

    execute(&mut cpu, &mut bus, decode(0x25E9_1CA3).unwrap()).unwrap();

    assert!(pred_bit(&cpu, 3, 0));
    assert!(!pred_bit(&cpu, 3, 8));
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.c());
    assert_eq!(cpu.regs.x(5), u64::MAX - 1);
}

#[test]
fn sve_while_signed_variants_use_signed_compare() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;
    cpu.regs.set_x(5, u32::MAX as u64);
    cpu.regs.set_x(2, 2);

    execute(&mut cpu, &mut bus, decode(0x2522_04A1).unwrap()).unwrap();
    assert!(pred_bit(&cpu, 1, 0));
    assert!(pred_bit(&cpu, 1, 1));
    assert!(pred_bit(&cpu, 1, 2));
    assert!(!pred_bit(&cpu, 1, 3));

    cpu.regs.set_x(5, 0);
    execute(&mut cpu, &mut bus, decode(0x2522_04B1).unwrap()).unwrap();
    for bit in 0..3 {
        assert!(pred_bit(&cpu, 1, bit));
    }
    assert!(!pred_bit(&cpu, 1, 3));
}

#[test]
fn sve_whilels_uses_unsigned_less_or_same() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;
    cpu.regs.set_x(5, 3);
    cpu.regs.set_x(2, 5);

    execute(&mut cpu, &mut bus, decode(0x2522_0CB1).unwrap()).unwrap();
    for bit in 0..3 {
        assert!(pred_bit(&cpu, 1, bit));
    }
    assert!(!pred_bit(&cpu, 1, 3));

    cpu.regs.set_x(5, u32::MAX as u64);
    cpu.regs.set_x(2, 1);
    execute(&mut cpu, &mut bus, decode(0x2522_0CB1).unwrap()).unwrap();
    assert_eq!(cpu.sve_pred[1], [0; 4]);
    assert!(cpu.pstate.z());
}
