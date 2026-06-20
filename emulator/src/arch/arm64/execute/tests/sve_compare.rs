use super::*;

#[test]
fn sve_unsigned_higher_same_compare_writes_predicates_and_flags() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x2598_E040).unwrap()).unwrap(); // ptrue p0.s, vl2
    set_z_word(&mut cpu, 26, 0, 10);
    set_z_word(&mut cpu, 26, 1, 5);
    set_z_word(&mut cpu, 26, 2, 9);
    set_z_word(&mut cpu, 24, 0, 9);
    set_z_word(&mut cpu, 24, 1, 6);
    set_z_word(&mut cpu, 24, 2, 1);
    execute(&mut cpu, &mut bus, decode(0x2498_0342).unwrap()).unwrap(); // cmphs p2.s, p0/z, z26.s, z24.s
    assert!(pred_bit(&cpu, 2, 0));
    assert!(!pred_bit(&cpu, 2, 4));
    assert!(!pred_bit(&cpu, 2, 8));
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.c());
    assert!(!cpu.pstate.v());

    execute(&mut cpu, &mut bus, decode(0x2518_E081).unwrap()).unwrap(); // ptrue p1.b, vl4
    set_z_byte(&mut cpu, 4, 0, 5);
    set_z_byte(&mut cpu, 4, 1, 3);
    set_z_byte(&mut cpu, 4, 2, 2);
    set_z_byte(&mut cpu, 4, 3, 0);
    execute(&mut cpu, &mut bus, decode(0x2420_C483).unwrap()).unwrap(); // cmphs p3.b, p1/z, z4.b, #3
    assert!(pred_bit(&cpu, 3, 0));
    assert!(pred_bit(&cpu, 3, 1));
    assert!(!pred_bit(&cpu, 3, 2));
    assert!(!pred_bit(&cpu, 3, 3));
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.c());
}

#[test]
fn sve_equal_and_higher_compare_forms_write_predicates() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x2598_E040).unwrap()).unwrap(); // ptrue p0.s, vl2
    set_z_word(&mut cpu, 0, 0, 7);
    set_z_word(&mut cpu, 0, 1, 8);
    set_z_word(&mut cpu, 30, 0, 7);
    set_z_word(&mut cpu, 30, 1, 9);
    execute(&mut cpu, &mut bus, decode(0x249E_A003).unwrap()).unwrap(); // cmpeq p3.s, p0/z, z0.s, z30.s
    assert!(pred_bit(&cpu, 3, 0));
    assert!(!pred_bit(&cpu, 3, 4));

    set_z_word(&mut cpu, 1, 0, 10);
    set_z_word(&mut cpu, 30, 0, 9);
    set_z_word(&mut cpu, 1, 1, 9);
    set_z_word(&mut cpu, 30, 1, 9);
    execute(&mut cpu, &mut bus, decode(0x249E_0032).unwrap()).unwrap(); // cmphi p2.s, p0/z, z1.s, z30.s
    assert!(pred_bit(&cpu, 2, 0));
    assert!(!pred_bit(&cpu, 2, 4));

    execute(&mut cpu, &mut bus, decode(0x2518_E080).unwrap()).unwrap(); // ptrue p0.b, vl4
    set_z_byte(&mut cpu, 1, 0, 0xFF);
    set_z_byte(&mut cpu, 1, 1, 0x7F);
    execute(&mut cpu, &mut bus, decode(0x251F_8030).unwrap()).unwrap(); // cmpne p0.b, p0/z, z1.b, #-1
    assert!(!pred_bit(&cpu, 0, 0));
    assert!(pred_bit(&cpu, 0, 1));
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
