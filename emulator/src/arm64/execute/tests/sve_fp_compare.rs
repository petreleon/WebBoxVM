use super::*;

#[test]
fn sve_fp_absolute_compare_writes_predicates_and_flags() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x25D8_E3E7).unwrap()).unwrap(); // ptrue p7.d
    set_z_elem(&mut cpu, 30, 0, 3.0f64.to_bits());
    set_z_elem(&mut cpu, 30, 1, (-4.0f64).to_bits());
    set_z_elem(&mut cpu, 30, 2, 2.0f64.to_bits());
    set_z_elem(&mut cpu, 30, 3, 1.0f64.to_bits());
    set_z_elem(&mut cpu, 27, 0, (-2.0f64).to_bits());
    set_z_elem(&mut cpu, 27, 1, 4.0f64.to_bits());
    set_z_elem(&mut cpu, 27, 2, (-5.0f64).to_bits());
    set_z_elem(&mut cpu, 27, 3, f64::NAN.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65DB_FFD3).unwrap()).unwrap(); // facgt p3.d, p7/z, z30.d, z27.d
    assert!(pred_bit(&cpu, 3, 0));
    assert!(!pred_bit(&cpu, 3, 8));
    assert!(!pred_bit(&cpu, 3, 16));
    assert!(!pred_bit(&cpu, 3, 24));
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.c());
    assert!(!cpu.pstate.v());

    execute(&mut cpu, &mut bus, decode(0x2598_E021).unwrap()).unwrap(); // ptrue p1.s, vl1
    set_z_word(&mut cpu, 27, 0, (-3.0f32).to_bits());
    set_z_word(&mut cpu, 27, 1, 10.0f32.to_bits());
    set_z_word(&mut cpu, 0, 0, 3.0f32.to_bits());
    set_z_word(&mut cpu, 0, 1, 1.0f32.to_bits());
    execute(&mut cpu, &mut bus, decode(0x6580_C772).unwrap()).unwrap(); // facge p2.s, p1/z, z27.s, z0.s
    assert!(pred_bit(&cpu, 2, 0));
    assert!(!pred_bit(&cpu, 2, 4));
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(!cpu.pstate.c());
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
