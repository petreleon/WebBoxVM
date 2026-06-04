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
}
