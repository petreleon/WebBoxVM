use super::*;

#[test]
fn sve_ftmad_uses_abs_second_operand_and_signed_coefficient_table() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    set_z_half(&mut cpu, 1, 0, 0x3C00);
    set_z_half(&mut cpu, 30, 0, 0xC000);
    execute(&mut cpu, &mut bus, decode(0x6550_83C1).unwrap()).unwrap();
    assert_eq!(z_half(&cpu, 1, 0), 0x4200);

    set_z_word(&mut cpu, 1, 0, 2.0f32.to_bits());
    set_z_word(&mut cpu, 30, 0, (-3.0f32).to_bits());
    execute(&mut cpu, &mut bus, decode(0x6591_83C1).unwrap()).unwrap();
    assert_eq!(z_word(&cpu, 1, 0), 5.5f32.to_bits());

    set_z_elem(&mut cpu, 1, 0, 2.0f64.to_bits());
    set_z_elem(&mut cpu, 30, 0, 3.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x65D1_83C1).unwrap()).unwrap();
    let expected = 2.0f64.mul_add(3.0, f64::from_bits(0xBFC5_5555_5555_5543));
    assert_eq!(z_elem(&cpu, 1, 0), expected.to_bits());
}

#[test]
fn sve_ftsmul_and_ftssel_use_quadrant_bits_as_integer_flags() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    set_z_half(&mut cpu, 2, 0, 0x4000);
    set_z_half(&mut cpu, 3, 0, 0x0001);
    execute(&mut cpu, &mut bus, decode(0x6543_0C41).unwrap()).unwrap();
    assert_eq!(z_half(&cpu, 1, 0), 0xC400);

    set_z_word(&mut cpu, 2, 0, (-3.0f32).to_bits());
    set_z_word(&mut cpu, 3, 0, 0);
    execute(&mut cpu, &mut bus, decode(0x6583_0C41).unwrap()).unwrap();
    assert_eq!(z_word(&cpu, 1, 0), 9.0f32.to_bits());

    set_z_elem(&mut cpu, 2, 0, 2.5f64.to_bits());
    set_z_elem(&mut cpu, 3, 0, 2);
    execute(&mut cpu, &mut bus, decode(0x04E3_B041).unwrap()).unwrap();
    assert_eq!(z_elem(&cpu, 1, 0), (-2.5f64).to_bits());

    set_z_word(&mut cpu, 2, 0, 7.0f32.to_bits());
    set_z_word(&mut cpu, 3, 0, 3);
    execute(&mut cpu, &mut bus, decode(0x04A3_B041).unwrap()).unwrap();
    assert_eq!(z_word(&cpu, 1, 0), (-1.0f32).to_bits());
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
