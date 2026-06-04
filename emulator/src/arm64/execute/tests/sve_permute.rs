use super::*;

#[test]
fn sve_zip_interleaves_low_and_high_dword_halves() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    for lane in 0..4 {
        set_z_elem(&mut cpu, 17, lane, 0x100 + lane as u64);
        set_z_elem(&mut cpu, 19, lane, 0x200 + lane as u64);
    }

    execute(&mut cpu, &mut bus, decode(0x05F3_6220).unwrap()).unwrap(); // zip1 z0.d, z17.d, z19.d
    assert_eq!(z_elem(&cpu, 0, 0), 0x100);
    assert_eq!(z_elem(&cpu, 0, 1), 0x200);
    assert_eq!(z_elem(&cpu, 0, 2), 0x101);
    assert_eq!(z_elem(&cpu, 0, 3), 0x201);

    execute(&mut cpu, &mut bus, decode(0x05F3_6624).unwrap()).unwrap(); // zip2 z4.d, z17.d, z19.d
    assert_eq!(z_elem(&cpu, 4, 0), 0x102);
    assert_eq!(z_elem(&cpu, 4, 1), 0x202);
    assert_eq!(z_elem(&cpu, 4, 2), 0x103);
    assert_eq!(z_elem(&cpu, 4, 3), 0x203);
    assert_eq!(cpu.simd[4], (0x202u128 << 64) | 0x102);
}

#[test]
fn sve_zip_word_form_uses_current_vector_length() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    for lane in 0..8 {
        set_z_word(&mut cpu, 0, lane, 0x10 + lane as u32);
        set_z_word(&mut cpu, 4, lane, 0x80 + lane as u32);
    }

    execute(&mut cpu, &mut bus, decode(0x05A4_6011).unwrap()).unwrap(); // zip1 z17.s, z0.s, z4.s
    assert_eq!(z_word(&cpu, 17, 0), 0x10);
    assert_eq!(z_word(&cpu, 17, 1), 0x80);
    assert_eq!(z_word(&cpu, 17, 6), 0x13);
    assert_eq!(z_word(&cpu, 17, 7), 0x83);
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}
