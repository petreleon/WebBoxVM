use super::*;

#[test]
fn sve_fp_abs_and_neg_merge_inactive_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x2558_E020).unwrap()).unwrap(); // ptrue p0.h, vl1
    set_z_half(&mut cpu, 1, 0, 0xBC00);
    set_z_half(&mut cpu, 1, 1, 0xC000);
    set_z_half(&mut cpu, 0, 0, 0x1111);
    set_z_half(&mut cpu, 0, 1, 0x2222);
    execute(&mut cpu, &mut bus, decode(0x045C_A020).unwrap()).unwrap(); // fabs z0.h, p0/m, z1.h
    assert_eq!(z_half(&cpu, 0, 0), 0x3C00);
    assert_eq!(z_half(&cpu, 0, 1), 0x2222);

    execute(&mut cpu, &mut bus, decode(0x25D8_E022).unwrap()).unwrap(); // ptrue p2.d, vl1
    set_z_elem(&mut cpu, 11, 0, 2.0f64.to_bits());
    set_z_elem(&mut cpu, 11, 1, (-3.0f64).to_bits());
    set_z_elem(&mut cpu, 10, 1, 7.0f64.to_bits());
    execute(&mut cpu, &mut bus, decode(0x04DD_A96A).unwrap()).unwrap(); // fneg z10.d, p2/m, z11.d
    assert_eq!(f64::from_bits(z_elem(&cpu, 10, 0)), -2.0);
    assert_eq!(f64::from_bits(z_elem(&cpu, 10, 1)), 7.0);
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
