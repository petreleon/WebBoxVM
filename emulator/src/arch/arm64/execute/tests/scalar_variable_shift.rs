use super::*;

#[test]
fn rorv_rotates_32_and_64_bit_register_values() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_x(1, 0x8000_0000_0000_0001);
    cpu.regs.set_x(2, 1);
    execute(&mut cpu, &mut bus, decode(0x9AC2_2C20).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0xC000_0000_0000_0000);

    cpu.regs.set_w(4, 0x8000_0001);
    cpu.regs.set_w(5, 33);
    execute(&mut cpu, &mut bus, decode(0x1AC5_2C83).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(3), 0xC000_0000);
}

#[test]
fn scalar_reverse_forms_swap_expected_byte_lanes() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_w(7, 0x1122_3344);
    execute(&mut cpu, &mut bus, decode(0x5AC0_04E6).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(6), 0x2211_4433);

    cpu.regs.set_x(5, 0x1122_3344_5566_7788);
    execute(&mut cpu, &mut bus, decode(0xDAC0_04A4).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(4), 0x2211_4433_6655_8877);

    cpu.regs.set_x(9, 0x1122_3344_AABB_CCDD);
    execute(&mut cpu, &mut bus, decode(0xDAC0_0928).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(8), 0x4433_2211_DDCC_BBAA);
}
