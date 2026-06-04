use super::*;

#[test]
fn cssc_scalar_bit_counts_update_destination_width() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_x(1, 0);
    execute(&mut cpu, &mut bus, decode(0xDAC0_1820).unwrap()).unwrap(); // ctz x0, x1
    assert_eq!(cpu.regs.x(0), 64);

    cpu.regs.set_x(1, 0x0000_0000_8000_0000);
    execute(&mut cpu, &mut bus, decode(0x5AC0_1820).unwrap()).unwrap(); // ctz w0, w1
    assert_eq!(cpu.regs.x(0), 31);

    cpu.regs.set_x(1, 0x8000_0000_0000_0010);
    execute(&mut cpu, &mut bus, decode(0xDAC0_1820).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 4);

    cpu.regs.set_x(1, u64::MAX);
    execute(&mut cpu, &mut bus, decode(0xDAC0_1C20).unwrap()).unwrap(); // cnt x0, x1
    assert_eq!(cpu.regs.x(0), 64);

    cpu.regs.set_x(1, 0xFFFF_0000_8000_0001);
    execute(&mut cpu, &mut bus, decode(0x5AC0_1C20).unwrap()).unwrap(); // cnt w0, w1
    assert_eq!(cpu.regs.x(0), 2);
}

#[test]
fn cssc_scalar_abs_wraps_in_destination_width() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_x(1, (-42i64) as u64);
    execute(&mut cpu, &mut bus, decode(0xDAC0_2020).unwrap()).unwrap(); // abs x0, x1
    assert_eq!(cpu.regs.x(0), 42);

    cpu.regs.set_x(1, i64::MIN as u64);
    execute(&mut cpu, &mut bus, decode(0xDAC0_2020).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), i64::MIN as u64);

    cpu.regs.set_x(1, 0xFFFF_FFFF_8000_0001);
    execute(&mut cpu, &mut bus, decode(0x5AC0_2020).unwrap()).unwrap(); // abs w0, w1
    assert_eq!(cpu.regs.x(0), 0x7FFF_FFFF);

    cpu.regs.set_x(1, 0xFFFF_FFFF_8000_0000);
    execute(&mut cpu, &mut bus, decode(0x5AC0_2020).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0x8000_0000);
}

#[test]
fn cssc_scalar_minmax_register_forms_use_destination_width() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_x(1, (-3i64) as u64);
    cpu.regs.set_x(2, 5);
    execute(&mut cpu, &mut bus, decode(0x9AC2_6020).unwrap()).unwrap(); // smax x0, x1, x2
    assert_eq!(cpu.regs.x(0), 5);
    execute(&mut cpu, &mut bus, decode(0x9AC2_6820).unwrap()).unwrap(); // smin x0, x1, x2
    assert_eq!(cpu.regs.x(0), (-3i64) as u64);

    cpu.regs.set_x(1, 0xFFFF_FFFF);
    cpu.regs.set_x(2, 1);
    execute(&mut cpu, &mut bus, decode(0x1AC2_6420).unwrap()).unwrap(); // umax w0, w1, w2
    assert_eq!(cpu.regs.x(0), 0xFFFF_FFFF);
    execute(&mut cpu, &mut bus, decode(0x1AC2_6C20).unwrap()).unwrap(); // umin w0, w1, w2
    assert_eq!(cpu.regs.x(0), 1);
}

#[test]
fn cssc_scalar_minmax_immediate_forms_decode_signedness() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_x(1, (-5i64) as u64);
    execute(&mut cpu, &mut bus, decode(0x91C3_FC20).unwrap()).unwrap(); // smax x0, x1, #-1
    assert_eq!(cpu.regs.x(0), u64::MAX);

    cpu.regs.set_x(1, 0);
    execute(&mut cpu, &mut bus, decode(0x11CA_0020).unwrap()).unwrap(); // smin w0, w1, #-128
    assert_eq!(cpu.regs.x(0), 0xFFFF_FF80);

    cpu.regs.set_x(1, 1);
    execute(&mut cpu, &mut bus, decode(0x91C7_FC20).unwrap()).unwrap(); // umax x0, x1, #255
    assert_eq!(cpu.regs.x(0), 255);

    cpu.regs.set_x(1, 4096);
    execute(&mut cpu, &mut bus, decode(0x11CF_FC20).unwrap()).unwrap(); // umin w0, w1, #255
    assert_eq!(cpu.regs.x(0), 255);
}
