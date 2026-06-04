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

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}

fn set_z_byte(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u8) {
    cpu.sve_z[reg][lane] = value;
    sync_simd_alias(cpu, reg);
}
