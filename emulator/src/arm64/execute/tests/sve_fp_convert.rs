use super::*;

#[test]
fn sve_fp_convert_updates_active_and_merges_inactive_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x2598_E020).unwrap()).unwrap(); // ptrue p0.s, vl1
    set_z_word(&mut cpu, 1, 0, (-7i32) as u32);
    set_z_word(&mut cpu, 0, 1, 0x7777_7777);
    execute(&mut cpu, &mut bus, decode(0x6594_A020).unwrap()).unwrap(); // scvtf z0.s, p0/m, z1.s
    assert_eq!(f32::from_bits(z_word(&cpu, 0, 0)), -7.0);
    assert_eq!(z_word(&cpu, 0, 1), 0x7777_7777);

    set_z_word(&mut cpu, 4, 0, (-2.9f32).to_bits());
    set_z_word(&mut cpu, 3, 1, 0x5555_5555);
    execute(&mut cpu, &mut bus, decode(0x659C_A083).unwrap()).unwrap(); // fcvtzs z3.s, p0/m, z4.s
    assert_eq!(z_word(&cpu, 3, 0), (-2i32) as u32);
    assert_eq!(z_word(&cpu, 3, 1), 0x5555_5555);

    execute(&mut cpu, &mut bus, decode(0x25D8_E021).unwrap()).unwrap(); // ptrue p1.d, vl1
    set_z_elem(&mut cpu, 1, 0, (-9i64) as u64);
    set_z_elem(&mut cpu, 0, 1, 0x1234);
    execute(&mut cpu, &mut bus, decode(0x65D6_A420).unwrap()).unwrap(); // scvtf z0.d, p1/m, z1.d
    assert_eq!(f64::from_bits(z_elem(&cpu, 0, 0)), -9.0);
    assert_eq!(z_elem(&cpu, 0, 1), 0x1234);

    set_z_elem(&mut cpu, 5, 0, (-2.9f64).to_bits());
    set_z_elem(&mut cpu, 6, 1, 0x5678);
    execute(&mut cpu, &mut bus, decode(0x65DE_A4A6).unwrap()).unwrap(); // fcvtzs z6.d, p1/m, z5.d
    assert_eq!(z_elem(&cpu, 6, 0), (-2i64) as u64);
    assert_eq!(z_elem(&cpu, 6, 1), 0x5678);
}

#[test]
fn sve_fp_convert_handles_unpacked_cross_size_forms() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;
    execute(&mut cpu, &mut bus, decode(0x25D8_E020).unwrap()).unwrap(); // ptrue p0.d, vl1

    set_z_elem(&mut cpu, 1, 0, 0xFFFF_FFFDu64);
    execute(&mut cpu, &mut bus, decode(0x65D0_A020).unwrap()).unwrap(); // scvtf z0.d, p0/m, z1.s
    assert_eq!(f64::from_bits(z_elem(&cpu, 0, 0)), -3.0);

    set_z_elem(&mut cpu, 1, 0, (-3i64) as u64);
    execute(&mut cpu, &mut bus, decode(0x65D4_A020).unwrap()).unwrap(); // scvtf z0.s, p0/m, z1.d
    assert_eq!(z_elem(&cpu, 0, 0), (-3.0f32).to_bits() as u64);

    set_z_elem(&mut cpu, 1, 0, (-3.25f32).to_bits() as u64);
    execute(&mut cpu, &mut bus, decode(0x65DC_A020).unwrap()).unwrap(); // fcvtzs z0.d, p0/m, z1.s
    assert_eq!(z_elem(&cpu, 0, 0), (-3i64) as u64);

    set_z_elem(&mut cpu, 1, 0, (-3.25f64).to_bits());
    execute(&mut cpu, &mut bus, decode(0x65D8_A020).unwrap()).unwrap(); // fcvtzs z0.s, p0/m, z1.d
    assert_eq!(z_elem(&cpu, 0, 0), (-3i64) as u64);
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
