use super::*;

#[test]
fn sve_xar_word_form_rotates_xor_with_current_vector_length() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    for lane in 0..8 {
        set_z_word(&mut cpu, 3, lane, 0x1234_0000 + lane as u32);
        set_z_word(&mut cpu, 0, lane, 0x00FF_00F0 + lane as u32);
    }

    execute(&mut cpu, &mut bus, decode(0x0470_3403).unwrap()).unwrap(); // xar z3.s, z3.s, z0.s, #16
    for lane in 0..8 {
        let lhs = 0x1234_0000 + lane as u32;
        let rhs = 0x00FF_00F0 + lane as u32;
        assert_eq!(z_word(&cpu, 3, lane), (lhs ^ rhs).rotate_right(16));
    }
}

#[test]
fn sve_xar_byte_rotation_stays_within_byte_lane() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;
    set_z_byte(&mut cpu, 5, 0, 0b1010_0001);
    set_z_byte(&mut cpu, 6, 0, 0b0001_0010);

    execute(&mut cpu, &mut bus, decode(0x042D_34C5).unwrap()).unwrap(); // xar z5.b, z5.b, z6.b, #3
    assert_eq!(z_byte(&cpu, 5, 0), 0b0111_0110);
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}

fn set_z_byte(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u8) {
    cpu.sve_z[reg][lane] = value;
    sync_simd_alias(cpu, reg);
}

fn z_byte(cpu: &Armv8Cpu, reg: usize, lane: usize) -> u8 {
    cpu.sve_z[reg][lane]
}
