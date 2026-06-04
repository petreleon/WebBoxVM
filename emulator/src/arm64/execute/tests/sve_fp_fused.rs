use super::*;

#[test]
fn sve_fp_fused_forms_apply_signs_and_predicates() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x2518_E081).unwrap()).unwrap(); // ptrue p1.s, vl4
    set_z_f32(&mut cpu, 4, [10.0, 20.0, 30.0, 40.0]);
    set_z_f32(&mut cpu, 5, [2.0, 3.0, 4.0, 5.0]);
    set_z_f32(&mut cpu, 6, [3.0, 4.0, 5.0, 6.0]);
    execute(&mut cpu, &mut bus, decode(0x65A6_04A4).unwrap()).unwrap(); // fmla
    assert_eq!(z_f32(&cpu, 4, 0), 16.0);
    assert_eq!(z_f32(&cpu, 4, 1), 20.0);

    set_z_f32(&mut cpu, 4, [10.0, 20.0, 30.0, 40.0]);
    execute(&mut cpu, &mut bus, decode(0x65A6_44A4).unwrap()).unwrap(); // fnmla
    assert_eq!(z_f32(&cpu, 4, 0), -16.0);
    assert_eq!(z_f32(&cpu, 4, 1), 20.0);

    set_z_f32(&mut cpu, 4, [10.0, 20.0, 30.0, 40.0]);
    execute(&mut cpu, &mut bus, decode(0x65A6_64A4).unwrap()).unwrap(); // fnmls
    assert_eq!(z_f32(&cpu, 4, 0), -4.0);
    assert_eq!(z_f32(&cpu, 4, 1), 20.0);

    execute(&mut cpu, &mut bus, decode(0x25D8_E022).unwrap()).unwrap(); // ptrue p2.d, vl1
    set_z_elem(&mut cpu, 7, 0, 10.0f64.to_bits());
    set_z_elem(&mut cpu, 7, 1, 20.0f64.to_bits());
    set_z_elem(&mut cpu, 8, 0, 3.0f64.to_bits());
    set_z_elem(&mut cpu, 9, 0, 100.0f64.to_bits());
    set_z_elem(&mut cpu, 9, 1, 200.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65E9_A907).unwrap()).unwrap(); // fmsb
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 0)), 70.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 1)), 20.0);

    set_z_elem(&mut cpu, 7, 0, 10.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65E9_C907).unwrap()).unwrap(); // fnmad
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 0)), -130.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 1)), 20.0);

    set_z_elem(&mut cpu, 7, 0, 10.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65E9_E907).unwrap()).unwrap(); // fnmsb
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 0)), -70.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 1)), 20.0);
}

#[test]
fn sve_fcmla_rotates_complex_pairs_and_preserves_inactive_components() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;

    cpu.sve_pred[1][0] = (1 << 0) | (1 << 4);
    set_z_f32(&mut cpu, 6, [10.0, 20.0, 30.0, 40.0]);
    set_z_f32(&mut cpu, 17, [2.0, 5.0, 0.0, 0.0]);
    set_z_f32(&mut cpu, 4, [3.0, 4.0, 0.0, 0.0]);
    execute(&mut cpu, &mut bus, decode(0x6444_4626).unwrap()).unwrap(); // #180
    assert_eq!(z_f32(&cpu, 6, 0), 4.0);
    assert_eq!(z_f32(&cpu, 6, 1), 12.0);
    assert_eq!(z_f32(&cpu, 6, 2), 30.0);
    assert_eq!(z_f32(&cpu, 6, 3), 40.0);

    set_z_f32(&mut cpu, 6, [10.0, 20.0, 30.0, 40.0]);
    execute(&mut cpu, &mut bus, decode(0x6444_2626).unwrap()).unwrap(); // #90
    assert_eq!(z_f32(&cpu, 6, 0), -10.0);
    assert_eq!(z_f32(&cpu, 6, 1), 35.0);
    assert_eq!(z_f32(&cpu, 6, 2), 30.0);
}

fn set_z_f32(cpu: &mut Armv8Cpu, reg: usize, values: [f32; 4]) {
    for (lane, value) in values.into_iter().enumerate() {
        let offset = lane * 4;
        cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_bits().to_le_bytes());
    }
    sync_simd_alias(cpu, reg);
}

fn z_f32(cpu: &Armv8Cpu, reg: usize, lane: usize) -> f32 {
    f32::from_bits(z_word(cpu, reg, lane))
}
