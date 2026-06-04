use super::*;

#[test]
fn sve_word_load_store_immediate_forms_transfer_active_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;
    let base = RAM_BASE + 0x7000;
    cpu.regs.set_x(5, base);

    execute(&mut cpu, &mut bus, decode(0x2598_E3E3).unwrap()).unwrap(); // ptrue p3.s
    for lane in 0..8 {
        bus.write(base + 32 + lane * 4, 4, 0x1000 + lane);
    }
    execute(&mut cpu, &mut bus, decode(0xA541_ACA7).unwrap()).unwrap(); // ld1w z7.s, p3/z, [x5, #1, mul vl]
    assert!((0..8).all(|lane| z_word(&cpu, 7, lane as usize) == 0x1000 + lane as u32));

    execute(&mut cpu, &mut bus, decode(0x2598_E043).unwrap()).unwrap(); // ptrue p3.s, vl2
    for lane in 0..8 {
        set_z_word(&mut cpu, 7, lane, 0x2000 + lane as u32);
        bus.write(base + 96 + lane as u64 * 4, 4, 0xDEAD_0000 + lane as u64);
    }
    cpu.regs.set_x(5, base + 64);
    execute(&mut cpu, &mut bus, decode(0xE541_ECA7).unwrap()).unwrap(); // st1w z7.s, p3, [x5, #1, mul vl]
    assert_eq!(bus.mem.read(base + 96, 4), Some(0x2000));
    assert_eq!(bus.mem.read(base + 100, 4), Some(0x2001));
    assert_eq!(bus.mem.read(base + 104, 4), Some(0xDEAD_0002));
}

#[test]
fn sve_ld1w_gather_forms_apply_scaled_offsets() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;
    let base = RAM_BASE + 0x8000;

    execute(&mut cpu, &mut bus, decode(0x2598_E3E0).unwrap()).unwrap(); // ptrue p0.s
    cpu.regs.set_x(3, base);
    for lane in 0..8 {
        set_z_word(&mut cpu, 6, lane, lane as u32);
        bus.write(base + lane as u64 * 4, 4, 0x3000 + lane as u64);
    }
    execute(&mut cpu, &mut bus, decode(0x8526_407D).unwrap()).unwrap(); // ld1w z29.s, p0/z, [x3, z6.s, uxtw #2]
    assert!((0..8).all(|lane| z_word(&cpu, 29, lane) == 0x3000 + lane as u32));

    cpu.regs.set_x(0, base + 0x104);
    for (lane, offset) in [0, -1, 1, 2].into_iter().enumerate() {
        set_z_word(&mut cpu, 28, lane, offset as i32 as u32);
    }
    for lane in 0..4 {
        bus.write(base + 0x100 + lane * 4, 4, 0x4000 + lane);
    }
    execute(&mut cpu, &mut bus, decode(0x857C_4001).unwrap()).unwrap(); // ld1w z1.s, p0/z, [x0, z28.s, sxtw #2]
    assert_eq!(z_word(&cpu, 1, 0), 0x4001);
    assert_eq!(z_word(&cpu, 1, 1), 0x4000);
    assert_eq!(z_word(&cpu, 1, 2), 0x4002);
}

#[test]
fn sve_word_load_store_dword_elements_zero_extend_and_truncate() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;
    let base = RAM_BASE + 0x9000;
    cpu.regs.set_x(0, base);
    execute(&mut cpu, &mut bus, decode(0x25D8_E3E3).unwrap()).unwrap(); // ptrue p3.d

    for lane in 0..4 {
        bus.write(base + lane * 4, 4, 0xA000 + lane);
    }
    execute(&mut cpu, &mut bus, decode(0xA560_AC00).unwrap()).unwrap(); // ld1w z0.d, p3/z, [x0]
    assert!((0..4).all(|lane| z_elem(&cpu, 0, lane as usize) == 0xA000 + lane));

    for lane in 0..4 {
        set_z_elem(&mut cpu, 0, lane as usize, 0xFFFF_0000_0000_5000 + lane);
    }
    execute(&mut cpu, &mut bus, decode(0xE560_EC00).unwrap()).unwrap(); // st1w z0.d, p3, [x0]
    assert_eq!(bus.mem.read(base, 4), Some(0x5000));
    assert_eq!(bus.mem.read(base + 4, 4), Some(0x5001));
}

#[test]
fn sve_halfword_load_forms_extend_and_zero_inactive_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;
    let base = RAM_BASE + 0xA000;

    cpu.regs.set_x(21, base + 0x100);
    cpu.sve_pred[4][0] = (1 << 0) | (1 << 2);
    bus.write(base + 0xB0, 2, 0x1234);
    bus.write(base + 0xB2, 2, 0xFEDC);
    bus.write(base + 0xB4, 2, 0xAAAA);
    execute(&mut cpu, &mut bus, decode(0xA4AB_B2B9).unwrap()).unwrap();
    assert_eq!(z_half(&cpu, 25, 0), 0x1234);
    assert_eq!(z_half(&cpu, 25, 1), 0xFEDC);
    assert_eq!(z_half(&cpu, 25, 2), 0);

    cpu.regs.set_x(22, base);
    cpu.sve_pred[4][0] = 1;
    set_z_elem(&mut cpu, 11, 0, 0x80);
    set_z_elem(&mut cpu, 11, 1, 0x82);
    bus.write(base + 0x80, 2, 0x00FE);
    bus.write(base + 0x82, 2, 0x00AA);
    execute(&mut cpu, &mut bus, decode(0xC4CB_D2D9).unwrap()).unwrap();
    assert_eq!(z_elem(&cpu, 25, 0), 0x00FE);
    assert_eq!(z_elem(&cpu, 25, 1), 0);

    cpu.sve_pred[4][0] = 1;
    set_z_word(&mut cpu, 20, 0, (base + 0x100) as u32);
    set_z_word(&mut cpu, 20, 1, (base + 0x104) as u32);
    cpu.regs.set_x(11, 2);
    bus.write(base + 0x102, 2, 0xFF80);
    bus.write(base + 0x106, 2, 0x007F);
    execute(&mut cpu, &mut bus, decode(0x848B_9299).unwrap()).unwrap();
    assert_eq!(z_word(&cpu, 25, 0), 0xFFFF_FF80);
    assert_eq!(z_word(&cpu, 25, 1), 0);
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
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&cpu.sve_z[reg][..16]);
    cpu.simd[reg] = u128::from_le_bytes(bytes);
}
