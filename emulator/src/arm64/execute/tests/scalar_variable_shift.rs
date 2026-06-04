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
