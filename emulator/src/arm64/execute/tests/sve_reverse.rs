use super::*;

#[test]
fn sve_revh_reverses_halfwords_inside_active_word_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x2598_E020).unwrap()).unwrap(); // ptrue p0.s, vl1
    set_z_word(&mut cpu, 3, 0, 0x1122_3344);
    set_z_word(&mut cpu, 3, 1, 0x5566_7788);
    execute(&mut cpu, &mut bus, decode(0x05A5_8063).unwrap()).unwrap(); // revh z3.s, p0/m, z3.s
    assert_eq!(z_word(&cpu, 3, 0), 0x3344_1122);
    assert_eq!(z_word(&cpu, 3, 1), 0x5566_7788);
    assert_eq!(cpu.simd[3] as u32, 0x3344_1122);
}

#[test]
fn sve_revh_reverses_halfwords_inside_active_dword_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x25D8_E020).unwrap()).unwrap(); // ptrue p0.d, vl1
    set_z_elem(&mut cpu, 3, 0, 0x1122_3344_5566_7788);
    set_z_elem(&mut cpu, 3, 1, 0x99aa_bbcc_ddee_ff00);
    execute(&mut cpu, &mut bus, decode(0x05E5_8063).unwrap()).unwrap(); // revh z3.d, p0/m, z3.d
    assert_eq!(z_elem(&cpu, 3, 0), 0x7788_5566_3344_1122);
    assert_eq!(z_elem(&cpu, 3, 1), 0x99aa_bbcc_ddee_ff00);
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
