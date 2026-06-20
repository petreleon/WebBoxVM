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

#[test]
fn sve_uzp_packs_even_word_and_odd_dword_elements() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    for lane in 0..8 {
        set_z_word(&mut cpu, 30, lane, 0x10 + lane as u32);
        set_z_word(&mut cpu, 27, lane, 0x80 + lane as u32);
    }

    execute(&mut cpu, &mut bus, decode(0x05BB_6BDE).unwrap()).unwrap(); // uzp1 z30.s, z30.s, z27.s
    assert_eq!(z_word(&cpu, 30, 0), 0x10);
    assert_eq!(z_word(&cpu, 30, 1), 0x12);
    assert_eq!(z_word(&cpu, 30, 3), 0x16);
    assert_eq!(z_word(&cpu, 30, 4), 0x80);
    assert_eq!(z_word(&cpu, 30, 7), 0x86);

    for lane in 0..4 {
        set_z_elem(&mut cpu, 0, lane, 0x30 + lane as u64);
        set_z_elem(&mut cpu, 31, lane, 0xA0 + lane as u64);
    }
    execute(&mut cpu, &mut bus, decode(0x05FF_6C18).unwrap()).unwrap(); // uzp2 z24.d, z0.d, z31.d
    assert_eq!(z_elem(&cpu, 24, 0), 0x31);
    assert_eq!(z_elem(&cpu, 24, 1), 0x33);
    assert_eq!(z_elem(&cpu, 24, 2), 0xA1);
    assert_eq!(z_elem(&cpu, 24, 3), 0xA3);
}

#[test]
fn sve_tbl_single_table_zeroes_out_of_range_indices() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;

    for lane in 0..16 {
        cpu.sve_z[3][lane] = 0x40 + lane as u8;
    }
    cpu.sve_z[31][..16].copy_from_slice(&[0, 1, 15, 16, 2, 99, 3, 14, 4, 5, 6, 7, 8, 9, 10, 11]);
    sync_simd_alias(&mut cpu, 3);
    sync_simd_alias(&mut cpu, 31);

    execute(&mut cpu, &mut bus, decode(0x053F_3063).unwrap()).unwrap();
    assert_eq!(
        &cpu.sve_z[3][..8],
        &[0x40, 0x41, 0x4F, 0, 0x42, 0, 0x43, 0x4E]
    );
    assert_eq!(cpu.simd[3] as u8, 0x40);
}

#[test]
fn sve_tbl_two_table_indexes_second_register() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;

    for lane in 0..8 {
        set_z_half(&mut cpu, 6, lane, 0x100 + lane as u16);
        set_z_half(&mut cpu, 7, lane, 0x200 + lane as u16);
    }
    for (lane, index) in [0, 7, 8, 15, 16, 2, 9, 6].into_iter().enumerate() {
        set_z_half(&mut cpu, 4, lane, index);
    }

    execute(&mut cpu, &mut bus, decode(0x0564_28C7).unwrap()).unwrap();
    assert_eq!(z_half(&cpu, 7, 0), 0x100);
    assert_eq!(z_half(&cpu, 7, 1), 0x107);
    assert_eq!(z_half(&cpu, 7, 2), 0x200);
    assert_eq!(z_half(&cpu, 7, 3), 0x207);
    assert_eq!(z_half(&cpu, 7, 4), 0);
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
    u16::from_le_bytes(cpu.sve_z[reg][offset..offset + 2].try_into().unwrap())
}
