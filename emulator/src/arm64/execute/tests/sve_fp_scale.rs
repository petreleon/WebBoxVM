use super::*;

#[test]
fn sve_fscale_uses_signed_raw_scale_and_preserves_inactive_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;
    cpu.sve_pred[3] = [1, 0, 0, 0];

    set_z_half(&mut cpu, 1, 0, 0x4000);
    set_z_half(&mut cpu, 31, 0, 0xFFFF);
    execute(&mut cpu, &mut bus, decode(0x6549_8FE1).unwrap()).unwrap();
    assert_eq!(z_half(&cpu, 1, 0), 0x3C00);

    set_z_word(&mut cpu, 1, 0, 1.5f32.to_bits());
    set_z_word(&mut cpu, 1, 1, 99.0f32.to_bits());
    set_z_word(&mut cpu, 31, 0, 2);
    set_z_word(&mut cpu, 31, 1, 12);
    execute(&mut cpu, &mut bus, decode(0x6589_8FE1).unwrap()).unwrap();
    assert_eq!(z_word(&cpu, 1, 0), 6.0f32.to_bits());
    assert_eq!(z_word(&cpu, 1, 1), 99.0f32.to_bits());

    set_z_elem(&mut cpu, 1, 0, 2.0f64.to_bits());
    set_z_elem(&mut cpu, 31, 0, (-2i64) as u64);
    execute(&mut cpu, &mut bus, decode(0x65C9_8FE1).unwrap()).unwrap();
    assert_eq!(z_elem(&cpu, 1, 0), 0.5f64.to_bits());
}

fn set_z_half(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u16) {
    let offset = lane * 2;
    cpu.sve_z[reg][offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}

fn z_half(cpu: &Armv8Cpu, reg: usize, lane: usize) -> u16 {
    let offset = lane * 2;
    let mut bytes = [0; 2];
    bytes.copy_from_slice(&cpu.sve_z[reg][offset..offset + 2]);
    u16::from_le_bytes(bytes)
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
