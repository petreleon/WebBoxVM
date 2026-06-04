use super::*;

#[test]
fn sve_fadd_immediates_update_active_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x2518_E083).unwrap()).unwrap(); // ptrue p3.b, vl4
    set_z_word(&mut cpu, 3, 0, 2.0f32.to_bits());
    set_z_word(&mut cpu, 3, 1, 4.0f32.to_bits());
    execute(&mut cpu, &mut bus, decode(0x6598_8C23).unwrap()).unwrap(); // fadd z3.s, p3/m, z3.s, #1
    assert_eq!(f32::from_bits(z_word(&cpu, 3, 0)), 3.0);
    assert_eq!(f32::from_bits(z_word(&cpu, 3, 1)), 4.0);

    execute(&mut cpu, &mut bus, decode(0x25D8_E024).unwrap()).unwrap(); // ptrue p4.d, vl1
    set_z_elem(&mut cpu, 4, 0, 8.0f64.to_bits());
    set_z_elem(&mut cpu, 4, 1, 10.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65D8_9004).unwrap()).unwrap(); // fadd z4.d, p4/m, z4.d, #0.5
    assert_eq!(f64::from_bits(z_elem(&cpu, 4, 0)), 8.5);
    assert_eq!(f64::from_bits(z_elem(&cpu, 4, 1)), 10.0);

    execute(&mut cpu, &mut bus, decode(0x2598_E020).unwrap()).unwrap(); // ptrue p0.s, vl1
    set_z_word(&mut cpu, 0, 0, 7.0f32.to_bits());
    set_z_word(&mut cpu, 0, 1, 9.0f32.to_bits());
    execute(&mut cpu, &mut bus, decode(0x6599_8020).unwrap()).unwrap(); // fsub z0.s, p0/m, z0.s, #1
    assert_eq!(f32::from_bits(z_word(&cpu, 0, 0)), 6.0);
    assert_eq!(f32::from_bits(z_word(&cpu, 0, 1)), 9.0);

    set_z_word(&mut cpu, 0, 0, 0.25f32.to_bits());
    execute(&mut cpu, &mut bus, decode(0x659B_8000).unwrap()).unwrap(); // fsubr z0.s, p0/m, z0.s, #0.5
    assert_eq!(f32::from_bits(z_word(&cpu, 0, 0)), 0.25);
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
