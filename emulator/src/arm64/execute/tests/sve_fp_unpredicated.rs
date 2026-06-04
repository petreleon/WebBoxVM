use super::*;

#[test]
fn sve_fp_unpredicated_add_sub_update_all_lanes() {
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
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
