use super::*;

#[test]
fn kernel_ldaddal_x_decrement_keeps_full_64_bit_source() {
    let (mut cpu, mut bus) = setup();
    let addr = RAM_BASE + 0x1800;
    cpu.regs.set_x(0, addr);
    bus.mem.write(addr, 8, 0x0000_0001_0000_0000);

    execute(&mut cpu, &mut bus, decode(0x9280_0021).unwrap()).unwrap(); // mov x1, #-2
    execute(&mut cpu, &mut bus, decode(0xF8E1_0001).unwrap()).unwrap(); // ldaddal x1, x1, [x0]

    assert_eq!(bus.mem.read(addr, 8), Some(0x0000_0000_FFFF_FFFE));
    assert_eq!(cpu.regs.x(1), 0x0000_0001_0000_0000);
}

#[test]
fn kernel_stadd_xzr_return_form_decrements_64_bit_counter() {
    let (mut cpu, mut bus) = setup();
    let addr = RAM_BASE + 0x1900;
    cpu.regs.set_x(0, addr);
    bus.mem.write(addr, 8, 0x0000_0001_0000_0000);

    execute(&mut cpu, &mut bus, decode(0x9280_0021).unwrap()).unwrap(); // mov x1, #-2
    execute(&mut cpu, &mut bus, decode(0xF821_001F).unwrap()).unwrap(); // stadd x1, [x0]

    assert_eq!(bus.mem.read(addr, 8), Some(0x0000_0000_FFFF_FFFE));
    assert_eq!(cpu.regs.x(1), u64::MAX - 1);
}

#[test]
fn kernel_zone_page_delta_sequence_sign_extends_before_atomic_add() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_w(20, 2);

    execute(&mut cpu, &mut bus, decode(0x4B14_03F4).unwrap()).unwrap(); // sub w20, wzr, w20
    execute(&mut cpu, &mut bus, decode(0x9340_7E94).unwrap()).unwrap(); // sxtw x20, w20
    execute(&mut cpu, &mut bus, decode(0xAA14_03E2).unwrap()).unwrap(); // mov x2, x20

    assert_eq!(cpu.regs.x(20), u64::MAX - 1);
    assert_eq!(cpu.regs.x(2), u64::MAX - 1);
}
