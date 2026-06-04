use super::*;

#[test]
fn sve_z_vector_forms_update_scalable_z_and_simd_alias() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    cpu.regs.set_x(1, 5);
    execute(&mut cpu, &mut bus, decode(0x05E0_383E).unwrap()).unwrap(); // dup z30.d, x1
    assert!((0..4).all(|lane| z_elem(&cpu, 30, lane) == 5));
    assert_eq!(cpu.simd[30], (5u128 << 64) | 5);

    cpu.regs.set_x(4, 7);
    execute(&mut cpu, &mut bus, decode(0x05E0_3898).unwrap()).unwrap(); // dup z24.d, x4
    execute(&mut cpu, &mut bus, decode(0x04F8_03D8).unwrap()).unwrap(); // add z24.d, z30.d, z24.d
    assert!((0..4).all(|lane| z_elem(&cpu, 24, lane) == 12));

    cpu.regs.set_x(0, 20);
    execute(&mut cpu, &mut bus, decode(0x05E0_381F).unwrap()).unwrap(); // dup z31.d, x0
    execute(&mut cpu, &mut bus, decode(0x04FE_07FE).unwrap()).unwrap(); // sub z30.d, z31.d, z30.d
    assert!((0..4).all(|lane| z_elem(&cpu, 30, lane) == 15));

    set_z_elem(&mut cpu, 27, 0, (10u64 << 32) | (u32::MAX - 1) as u64);
    set_z_elem(&mut cpu, 26, 0, (5u64 << 32) | 4);
    execute(&mut cpu, &mut bus, decode(0x04BA_177B).unwrap()).unwrap(); // uqadd z27.s, z27.s, z26.s
    assert_eq!(z_word(&cpu, 27, 0), u32::MAX);
    assert_eq!(z_word(&cpu, 27, 1), 15);

    set_z_elem(&mut cpu, 28, 0, 0x1111);
    set_z_elem(&mut cpu, 28, 1, 0x2222);
    set_z_elem(&mut cpu, 28, 2, 0x3333);
    set_z_elem(&mut cpu, 28, 3, 0x4444);
    execute(&mut cpu, &mut bus, decode(0x0420_BF9A).unwrap()).unwrap(); // movprfx z26, z28
    assert!((0..4).all(|lane| z_elem(&cpu, 26, lane) == z_elem(&cpu, 28, lane)));

    execute(&mut cpu, &mut bus, decode(0x25D8_E023).unwrap()).unwrap(); // ptrue p3.d, vl1
    set_z_elem(&mut cpu, 1, 0, 0xAAAA);
    set_z_elem(&mut cpu, 1, 1, 0xBBBB);
    set_z_elem(&mut cpu, 29, 0, u64::MAX);
    set_z_elem(&mut cpu, 29, 1, u64::MAX);
    set_z_elem(&mut cpu, 29, 2, u64::MAX);
    execute(&mut cpu, &mut bus, decode(0x04D0_2C3D).unwrap()).unwrap(); // movprfx z29.d, p3/z, z1.d
    assert_eq!(z_elem(&cpu, 29, 0), 0xAAAA);
    assert_eq!(z_elem(&cpu, 29, 1), 0);
    assert_eq!(z_elem(&cpu, 29, 2), 0);

    set_z_elem(&mut cpu, 0, 0, 0x00FF);
    set_z_elem(&mut cpu, 0, 1, 0x0F0F);
    execute(&mut cpu, &mut bus, decode(0x0460_301B).unwrap()).unwrap(); // orr z27.d, z0.d, z0.d
    assert_eq!(z_elem(&cpu, 27, 0), 0x00FF);
    assert_eq!(z_elem(&cpu, 27, 1), 0x0F0F);
    execute(&mut cpu, &mut bus, decode(0x04A0_3000).unwrap()).unwrap(); // eor z0.d, z0.d, z0.d
    assert!((0..4).all(|lane| z_elem(&cpu, 0, lane) == 0));

    execute(&mut cpu, &mut bus, decode(0x25D8_E020).unwrap()).unwrap(); // ptrue p0.d, vl1
    cpu.regs.set_x(2, 0xAAAA);
    cpu.regs.set_x(3, 0xBBBB);
    execute(&mut cpu, &mut bus, decode(0x05E0_3842).unwrap()).unwrap(); // dup z2.d, x2
    execute(&mut cpu, &mut bus, decode(0x05E0_387F).unwrap()).unwrap(); // dup z31.d, x3
    execute(&mut cpu, &mut bus, decode(0x05FF_C040).unwrap()).unwrap(); // sel z0.d, p0, z2.d, z31.d
    assert_eq!(z_elem(&cpu, 0, 0), 0xAAAA);
    assert!((1..4).all(|lane| z_elem(&cpu, 0, lane) == 0xBBBB));

    execute(&mut cpu, &mut bus, decode(0x2518_E081).unwrap()).unwrap(); // ptrue p1.b, vl4
    set_z_f32(&mut cpu, 4, [1.0, 2.0, 3.0, 4.0]);
    set_z_f32(&mut cpu, 5, [0.5, 1.5, 2.5, 3.5]);
    execute(&mut cpu, &mut bus, decode(0x6580_84A4).unwrap()).unwrap(); // fadd z4.s, p1/m, z4.s, z5.s
    assert_eq!(z_f32(&cpu, 4, 0), 1.5);
    assert_eq!(z_f32(&cpu, 4, 3), 4.0);

    execute(&mut cpu, &mut bus, decode(0x25D8_E022).unwrap()).unwrap(); // ptrue p2.d, vl1
    set_z_elem(&mut cpu, 7, 0, 10.0f64.to_bits());
    set_z_elem(&mut cpu, 7, 1, 20.0f64.to_bits());
    set_z_elem(&mut cpu, 8, 0, 3.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65C3_8907).unwrap()).unwrap(); // fsubr z7.d, p2/m, z7.d, z8.d
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 0)), -7.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 1)), 20.0);

    set_z_elem(&mut cpu, 28, 0, 5.0f64.to_bits());
    set_z_elem(&mut cpu, 28, 1, 6.0f64.to_bits());
    set_z_elem(&mut cpu, 31, 0, 7.0f64.to_bits());
    set_z_elem(&mut cpu, 31, 1, 8.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65DC_0BFF).unwrap()).unwrap(); // fmul z31.d, z31.d, z28.d
    assert_eq!(f64::from_bits(z_elem(&cpu, 31, 0)), 35.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 31, 1)), 48.0);

    execute(&mut cpu, &mut bus, decode(0x25D8_E023).unwrap()).unwrap(); // ptrue p3.d, vl1
    set_z_elem(&mut cpu, 1, 0, 9.0f64.to_bits());
    set_z_elem(&mut cpu, 1, 1, 10.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65DA_8C21).unwrap()).unwrap(); // fmul z1.d, p3/m, z1.d, #2
    assert_eq!(f64::from_bits(z_elem(&cpu, 1, 0)), 18.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 1, 1)), 10.0);

    execute(&mut cpu, &mut bus, decode(0x2579_CE00).unwrap()).unwrap(); // fdup z0.h, #1.0
    assert!((0..16).all(|lane| z_half(&cpu, 0, lane) == 0x3C00));

    execute(&mut cpu, &mut bus, decode(0x25B9_DC05).unwrap()).unwrap(); // fdup z5.s, #-0.5
    assert!((0..8).all(|lane| z_word(&cpu, 5, lane) == 0xBF00_0000));
    assert_eq!(cpu.simd[5], 0xBF00_0000_BF00_0000_BF00_0000_BF00_0000);

    execute(&mut cpu, &mut bus, decode(0x25F9_C01D).unwrap()).unwrap(); // fdup z29.d, #2.0
    assert!((0..4).all(|lane| z_elem(&cpu, 29, lane) == 2.0f64.to_bits()));

    set_z_f32(&mut cpu, 4, [10.0, 20.0, 30.0, 40.0]);
    set_z_f32(&mut cpu, 5, [2.0, 3.0, 4.0, 5.0]);
    set_z_f32(&mut cpu, 6, [3.0, 4.0, 5.0, 6.0]);
    execute(&mut cpu, &mut bus, decode(0x65A6_04A4).unwrap()).unwrap(); // fmla z4.s, p1/m, z5.s, z6.s
    assert_eq!(z_f32(&cpu, 4, 0), 16.0);
    assert_eq!(z_f32(&cpu, 4, 1), 20.0);

    set_z_elem(&mut cpu, 7, 0, 10.0f64.to_bits());
    set_z_elem(&mut cpu, 7, 1, 20.0f64.to_bits());
    set_z_elem(&mut cpu, 8, 0, 3.0f64.to_bits());
    set_z_elem(&mut cpu, 9, 0, 100.0f64.to_bits());
    set_z_elem(&mut cpu, 9, 1, 200.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65E9_A907).unwrap()).unwrap(); // fmsb z7.d, p2/m, z8.d, z9.d
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 0)), 70.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 1)), 20.0);

    set_z_f32(&mut cpu, 4, [10.0, 20.0, 30.0, 40.0]);
    set_z_f32(&mut cpu, 5, [2.0, 3.0, 4.0, 5.0]);
    set_z_f32(&mut cpu, 6, [10.0, 20.0, 30.0, 40.0]);
    execute(&mut cpu, &mut bus, decode(0x64B6_00A4).unwrap()).unwrap(); // fmla z4.s, z5.s, z6.s[2]
    assert_eq!(z_f32(&cpu, 4, 0), 70.0);
    assert_eq!(z_f32(&cpu, 4, 1), 110.0);

    for lane in 0..4 {
        set_z_elem(&mut cpu, 8, lane, ((lane + 2) as f64).to_bits());
        set_z_elem(&mut cpu, 9, lane, ((lane + 1) as f64 * 10.0).to_bits());
    }
    execute(&mut cpu, &mut bus, decode(0x64F9_2107).unwrap()).unwrap(); // fmul z7.d, z8.d, z9.d[1]
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 0)), 40.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 7, 2)), 160.0);
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

fn z_half(cpu: &Armv8Cpu, reg: usize, lane: usize) -> u16 {
    let offset = lane * 2;
    let mut bytes = [0; 2];
    bytes.copy_from_slice(&cpu.sve_z[reg][offset..offset + 2]);
    u16::from_le_bytes(bytes)
}
