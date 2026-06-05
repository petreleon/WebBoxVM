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

    cpu.sve_vl_bytes = 32;
    cpu.regs.sp = 0x1000;
    execute(&mut cpu, &mut bus, decode(0x047F_57FF).unwrap()).unwrap(); // addpl sp, sp, #-1
    assert_eq!(cpu.regs.sp, 0x0ffc);

    cpu.sme_svl_bytes = 32;
    execute(&mut cpu, &mut bus, decode(0x04BF_5210).unwrap()).unwrap(); // rdvl x16, #16
    assert_eq!(cpu.regs.x(16), 0x200);

    execute(&mut cpu, &mut bus, decode(0x04BF_5A10).unwrap()).unwrap(); // rdsvl x16, #16
    assert_eq!(cpu.regs.x(16), 0x200);

    cpu.regs.set_x(16, 0x2000);
    execute(&mut cpu, &mut bus, decode(0x0430_5A10).unwrap()).unwrap(); // addsvl x16, x16, #16
    assert_eq!(cpu.regs.x(16), 0x2200);

    cpu.regs.set_x(16, 0x2000);
    execute(&mut cpu, &mut bus, decode(0x0470_5A10).unwrap()).unwrap(); // addspl x16, x16, #16
    assert_eq!(cpu.regs.x(16), 0x2040);

    cpu.sve_vl_bytes = 32;
    cpu.regs.set_x(17, 100);
    execute(&mut cpu, &mut bus, decode(0x0431_E3F1).unwrap()).unwrap(); // incb x17, all, mul #2
    assert_eq!(cpu.regs.x(17), 164);

    cpu.regs.set_x(18, 100);
    execute(&mut cpu, &mut bus, decode(0x04F1_E7F2).unwrap()).unwrap(); // decd x18, all, mul #2
    assert_eq!(cpu.regs.x(18), 92);

    cpu.regs.sp = 0x4000;
    execute(&mut cpu, &mut bus, decode(0x0431_E3FF).unwrap()).unwrap(); // incb xzr, all, mul #2
    assert_eq!(cpu.regs.sp, 0x4000);

    cpu.sve_pred[7] = [0b1 | (1 << 16), 0, 0, 0];
    cpu.regs.set_x(1, 20);
    execute(&mut cpu, &mut bus, decode(0x25EC_88E1).unwrap()).unwrap(); // incp x1, p7.d
    assert_eq!(cpu.regs.x(1), 22);

    execute(&mut cpu, &mut bus, decode(0x25ED_88E1).unwrap()).unwrap(); // decp x1, p7.d
    assert_eq!(cpu.regs.x(1), 20);

    let mut z = [0; 256];
    z[..8].copy_from_slice(&10u64.to_le_bytes());
    z[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
    z[16..24].copy_from_slice(&40u64.to_le_bytes());
    cpu.sve_z[2] = z;
    cpu.simd[2] = u128::from_le_bytes(z[..16].try_into().unwrap());
    execute(&mut cpu, &mut bus, decode(0x25EC_80E2).unwrap()).unwrap(); // incp z2.d, p7.d
    assert_eq!(
        u64::from_le_bytes(cpu.sve_z[2][..8].try_into().unwrap()),
        12
    );
    assert_eq!(
        u64::from_le_bytes(cpu.sve_z[2][8..16].try_into().unwrap()),
        1
    );
    assert_eq!(
        u64::from_le_bytes(cpu.sve_z[2][16..24].try_into().unwrap()),
        42
    );

    execute(&mut cpu, &mut bus, decode(0x25ED_80E2).unwrap()).unwrap(); // decp z2.d, p7.d
    assert_eq!(
        u64::from_le_bytes(cpu.sve_z[2][..8].try_into().unwrap()),
        10
    );
    assert_eq!(
        u64::from_le_bytes(cpu.sve_z[2][8..16].try_into().unwrap()),
        u64::MAX
    );

    cpu.sve_z[3] = z;
    cpu.simd[3] = u128::from_le_bytes(z[..16].try_into().unwrap());
    execute(&mut cpu, &mut bus, decode(0x04F1_C3E3).unwrap()).unwrap(); // incd z3.d, all, mul #2
    assert_eq!(
        u64::from_le_bytes(cpu.sve_z[3][..8].try_into().unwrap()),
        18
    );
    assert_eq!(
        u64::from_le_bytes(cpu.sve_z[3][8..16].try_into().unwrap()),
        7
    );
    assert_eq!(
        u64::from_le_bytes(cpu.sve_z[3][16..24].try_into().unwrap()),
        48
    );

    execute(&mut cpu, &mut bus, decode(0x04F1_C7E3).unwrap()).unwrap(); // decd z3.d, all, mul #2
    assert_eq!(
        u64::from_le_bytes(cpu.sve_z[3][..8].try_into().unwrap()),
        10
    );
    assert_eq!(
        u64::from_le_bytes(cpu.sve_z[3][8..16].try_into().unwrap()),
        u64::MAX
    );
}
