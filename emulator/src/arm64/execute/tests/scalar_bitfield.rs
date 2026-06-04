use super::*;

#[test]
fn scalar_bitfield_extracts_signs_and_inserts() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_w(7, 0x0000_8001);
    execute(&mut cpu, &mut bus, decode(0x1300_3CE6).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(6), 0xFFFF_8001);

    cpu.regs.set_w(9, 0x1234_5678);
    execute(&mut cpu, &mut bus, decode(0x5308_3D28).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(8), 0x56);

    cpu.regs.set_x(4, 0xFFFF_FFFF_FFFF_FF00);
    cpu.regs.set_x(5, 0x1234);
    execute(&mut cpu, &mut bus, decode(0xB348_3CA4).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(4), 0xFFFF_FFFF_FFFF_FF12);
}
