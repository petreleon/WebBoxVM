use super::*;

#[test]
fn sve_fp_div_and_reverse_merge_inactive_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x2598_E020).unwrap()).unwrap(); // ptrue p0.s, vl1
    set_z_word(&mut cpu, 0, 0, 8.0f32.to_bits());
    set_z_word(&mut cpu, 0, 1, 12.0f32.to_bits());
    set_z_word(&mut cpu, 1, 0, 2.0f32.to_bits());
    set_z_word(&mut cpu, 1, 1, 3.0f32.to_bits());
    execute(&mut cpu, &mut bus, decode(0x658D_8020).unwrap()).unwrap(); // fdiv z0.s, p0/m, z0.s, z1.s
    assert_eq!(f32::from_bits(z_word(&cpu, 0, 0)), 4.0);
    assert_eq!(f32::from_bits(z_word(&cpu, 0, 1)), 12.0);

    execute(&mut cpu, &mut bus, decode(0x25D8_E021).unwrap()).unwrap(); // ptrue p1.d, vl1
    set_z_elem(&mut cpu, 8, 0, 4.0f64.to_bits());
    set_z_elem(&mut cpu, 8, 1, 6.0f64.to_bits());
    set_z_elem(&mut cpu, 9, 0, 20.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65CC_8528).unwrap()).unwrap(); // fdivr z8.d, p1/m, z8.d, z9.d
    assert_eq!(f64::from_bits(z_elem(&cpu, 8, 0)), 5.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 8, 1)), 6.0);
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
