use super::*;

#[test]
fn sve_dup_immediate_and_indexed_fill_scalable_vector() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x2538_DFE1).unwrap()).unwrap(); // dup z1.b, #-1
    assert!((0..32).all(|lane| z_byte(&cpu, 1, lane) == 0xFF));

    execute(&mut cpu, &mut bus, decode(0x2578_E024).unwrap()).unwrap(); // dup z4.h, #1, lsl #8
    assert!((0..16).all(|lane| z_half_local(&cpu, 4, lane) == 0x0100));

    execute(&mut cpu, &mut bus, decode(0x25B8_FFC5).unwrap()).unwrap(); // dup z5.s, #-2, lsl #8
    assert!((0..8).all(|lane| z_word(&cpu, 5, lane) == 0xFFFF_FE00));

    set_z_word(&mut cpu, 13, 5, 0xDEAD_BEEF);
    execute(&mut cpu, &mut bus, decode(0x056C_21AC).unwrap()).unwrap(); // dup z12.s, z13.s[5]
    assert!((0..8).all(|lane| z_word(&cpu, 12, lane) == 0xDEAD_BEEF));

    set_z_elem(&mut cpu, 23, 7, 0xFFFF);
    execute(&mut cpu, &mut bus, decode(0x05F8_22F6).unwrap()).unwrap(); // dup z22.d, z23.d[7]
    assert!((0..4).all(|lane| z_elem(&cpu, 22, lane) == 0));
}

#[test]
fn sve_cpy_immediate_updates_active_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x25D8_E022).unwrap()).unwrap(); // ptrue p2.d, vl1
    set_z_elem(&mut cpu, 31, 0, 0xAAAA);
    set_z_elem(&mut cpu, 31, 1, 0xBBBB);
    execute(&mut cpu, &mut bus, decode(0x05D2_401F).unwrap()).unwrap(); // cpy z31.d, p2/m, #0
    assert_eq!(z_elem(&cpu, 31, 0), 0);
    assert_eq!(z_elem(&cpu, 31, 1), 0xBBBB);

    set_z_elem(&mut cpu, 31, 0, 0xAAAA);
    set_z_elem(&mut cpu, 31, 1, 0xBBBB);
    execute(&mut cpu, &mut bus, decode(0x05D2_001F).unwrap()).unwrap(); // mov z31.d, p2/z, #0
    assert_eq!(z_elem(&cpu, 31, 0), 0);
    assert_eq!(z_elem(&cpu, 31, 1), 0);
}

#[test]
fn sve_cpy_gpr_updates_active_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;

    execute(&mut cpu, &mut bus, decode(0x25D8_E021).unwrap()).unwrap(); // ptrue p1.d, vl1
    cpu.regs.set_x(0, 0x1234_5678_9ABC_DEF0);
    set_z_elem(&mut cpu, 26, 0, 0xAAAA);
    set_z_elem(&mut cpu, 26, 1, 0xBBBB);
    execute(&mut cpu, &mut bus, decode(0x05E8_A41A).unwrap()).unwrap(); // cpy z26.d, p1/m, x0
    assert_eq!(z_elem(&cpu, 26, 0), 0x1234_5678_9ABC_DEF0);
    assert_eq!(z_elem(&cpu, 26, 1), 0xBBBB);
}

fn set_z_word(cpu: &mut Armv8Cpu, reg: usize, lane: usize, value: u32) {
    let offset = lane * 4;
    cpu.sve_z[reg][offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    sync_simd_alias(cpu, reg);
}

fn z_byte(cpu: &Armv8Cpu, reg: usize, lane: usize) -> u8 {
    cpu.sve_z[reg][lane]
}

fn z_half_local(cpu: &Armv8Cpu, reg: usize, lane: usize) -> u16 {
    let offset = lane * 2;
    u16::from_le_bytes(cpu.sve_z[reg][offset..offset + 2].try_into().unwrap())
}
