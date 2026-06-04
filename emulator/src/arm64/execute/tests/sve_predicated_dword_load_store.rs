use super::*;

#[test]
fn sve_predicated_dword_load_store_forms_transfer_active_lanes() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;
    let base = RAM_BASE + 0x3000;

    cpu.regs.set_x(0, base);
    execute(&mut cpu, &mut bus, decode(0x25D8_E3E3).unwrap()).unwrap(); // ptrue p3.d
    bus.write(base + 64, 8, 0x1111_2222_3333_4444);
    execute(&mut cpu, &mut bus, decode(0x85C8_EC07).unwrap()).unwrap(); // ld1rd { z7.d }, p3/z, [x0, #0x40]
    assert!((0..4).all(|lane| z_elem(&cpu, 7, lane) == 0x1111_2222_3333_4444));

    execute(&mut cpu, &mut bus, decode(0x25D8_E023).unwrap()).unwrap(); // ptrue p3.d, vl1
    bus.write(base + 32, 8, 0xAAAA_BBBB_CCCC_DDDD);
    bus.write(base + 40, 8, 0x5555_6666_7777_8888);
    execute(&mut cpu, &mut bus, decode(0xA582_2C00).unwrap()).unwrap(); // ld1rqd { z0.d }, p3/z, [x0, #0x20]
    assert_eq!(z_elem(&cpu, 0, 0), 0xAAAA_BBBB_CCCC_DDDD);
    assert_eq!(z_elem(&cpu, 0, 1), 0);
    assert_eq!(z_elem(&cpu, 0, 2), 0xAAAA_BBBB_CCCC_DDDD);
    assert_eq!(z_elem(&cpu, 0, 3), 0);

    cpu.regs.set_x(3, base + 0x100);
    execute(&mut cpu, &mut bus, decode(0x25D8_E3E0).unwrap()).unwrap(); // ptrue p0.d
    for lane in 0..4 {
        set_z_elem(&mut cpu, 6, lane, lane as u64);
        bus.write(base + 0x100 + lane as u64 * 8, 8, 0x1000 + lane as u64);
    }
    execute(&mut cpu, &mut bus, decode(0xC5E6_C07D).unwrap()).unwrap(); // ld1d { z29.d }, p0/z, [x3, z6.d, lsl #3]
    assert!((0..4).all(|lane| z_elem(&cpu, 29, lane) == 0x1000 + lane as u64));

    cpu.regs.set_x(0, base + 0x200);
    execute(&mut cpu, &mut bus, decode(0x25D8_E3E3).unwrap()).unwrap(); // ptrue p3.d
    for lane in 0..4 {
        bus.write(base + 0x200 + 32 + lane as u64 * 8, 8, 0x2000 + lane as u64);
    }
    execute(&mut cpu, &mut bus, decode(0xA5E1_AC00).unwrap()).unwrap(); // ld1d { z0.d }, p3/z, [x0, #0x1, mul vl]
    assert!((0..4).all(|lane| z_elem(&cpu, 0, lane) == 0x2000 + lane as u64));

    execute(&mut cpu, &mut bus, decode(0x25D8_E023).unwrap()).unwrap(); // ptrue p3.d, vl1
    cpu.regs.set_x(0, base + 0x300);
    for lane in 0..4 {
        set_z_elem(&mut cpu, 0, lane, 0x3000 + lane as u64);
        bus.write(base + 0x300 + lane as u64 * 8, 8, 0xDEAD_0000 + lane as u64);
    }
    execute(&mut cpu, &mut bus, decode(0xE5E0_EC00).unwrap()).unwrap(); // st1d { z0.d }, p3, [x0]
    assert_eq!(bus.mem.read(base + 0x300, 8), Some(0x3000));
    assert_eq!(bus.mem.read(base + 0x308, 8), Some(0xDEAD_0001));
    assert_eq!(bus.mem.read(base + 0x310, 8), Some(0xDEAD_0002));
    assert_eq!(bus.mem.read(base + 0x318, 8), Some(0xDEAD_0003));
}
