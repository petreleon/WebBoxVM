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

#[test]
fn sve_fcpy_immediates_update_active_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x25D8_E023).unwrap()).unwrap(); // ptrue p3.d, vl1
    set_z_elem(&mut cpu, 31, 0, 8.0f64.to_bits());
    set_z_elem(&mut cpu, 31, 1, 10.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x05D3_CE1F).unwrap()).unwrap(); // fcpy z31.d, p3/m, #1
    assert_eq!(f64::from_bits(z_elem(&cpu, 31, 0)), 1.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 31, 1)), 10.0);

    execute(&mut cpu, &mut bus, decode(0x2558_E020).unwrap()).unwrap(); // ptrue p0.h, vl1
    set_z_half(&mut cpu, 29, 0, 0);
    set_z_half(&mut cpu, 29, 1, 0x2222);
    execute(&mut cpu, &mut bus, decode(0x0552_CE1D).unwrap()).unwrap(); // fcpy z29.h, p2/m, #1
    assert_eq!(z_half(&cpu, 29, 0), 0);

    execute(&mut cpu, &mut bus, decode(0x2558_E022).unwrap()).unwrap(); // ptrue p2.h, vl1
    execute(&mut cpu, &mut bus, decode(0x0552_CE1D).unwrap()).unwrap(); // fcpy z29.h, p2/m, #1
    assert_eq!(z_half(&cpu, 29, 0), 0x3C00);
    assert_eq!(z_half(&cpu, 29, 1), 0x2222);
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
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
