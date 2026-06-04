use super::*;

#[test]
fn sve_fp_binary_forms_update_active_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    for lane in 0..8 {
        set_z_word(&mut cpu, 31, lane, (lane as f32 + 1.0).to_bits());
        set_z_word(&mut cpu, 30, lane, 2.0f32.to_bits());
    }
    execute(&mut cpu, &mut bus, decode(0x659E_03FE).unwrap()).unwrap(); // fadd z30.s, z31.s, z30.s
    assert_eq!(f32::from_bits(z_word(&cpu, 30, 0)), 3.0);
    assert_eq!(f32::from_bits(z_word(&cpu, 30, 7)), 10.0);

    for lane in 0..8 {
        set_z_word(&mut cpu, 31, lane, (lane as f32 + 10.0).to_bits());
        set_z_word(&mut cpu, 30, lane, 1.0f32.to_bits());
    }
    execute(&mut cpu, &mut bus, decode(0x659E_07FE).unwrap()).unwrap(); // fsub z30.s, z31.s, z30.s
    assert_eq!(f32::from_bits(z_word(&cpu, 30, 0)), 9.0);
    assert_eq!(f32::from_bits(z_word(&cpu, 30, 7)), 16.0);

    cpu.sve_pred[0][0] = (1 << 0) | (1 << 8);
    for lane in 0..4 {
        set_z_word(
            &mut cpu,
            30,
            lane,
            [10.0f32, -0.0, -5.0, 8.0][lane].to_bits(),
        );
        set_z_word(&mut cpu, 31, lane, [3.0f32, 0.0, -9.0, 1.0][lane].to_bits());
    }
    execute(&mut cpu, &mut bus, decode(0x6587_83FE).unwrap()).unwrap(); // fmin z30.s, p0/m, z30.s, z31.s
    assert_eq!(f32::from_bits(z_word(&cpu, 30, 0)), 3.0);
    assert_eq!(z_word(&cpu, 30, 1), (-0.0f32).to_bits());
    assert_eq!(f32::from_bits(z_word(&cpu, 30, 2)), -9.0);
    assert_eq!(f32::from_bits(z_word(&cpu, 30, 3)), 8.0);

    cpu.sve_pred[0][0] = 1;
    set_z_elem(&mut cpu, 30, 0, 12.5f64.to_bits());
    set_z_elem(&mut cpu, 29, 0, (-2.25f64).to_bits());
    set_z_elem(&mut cpu, 30, 1, 99.0f64.to_bits());
    set_z_elem(&mut cpu, 29, 1, 1.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65C8_83BE).unwrap()).unwrap(); // fabd z30.d, p0/m, z30.d, z29.d
    assert_eq!(f64::from_bits(z_elem(&cpu, 30, 0)), 14.75);
    assert_eq!(f64::from_bits(z_elem(&cpu, 30, 1)), 99.0);
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
