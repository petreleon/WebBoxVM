use super::*;

#[test]
fn stxp_pair_store_can_use_xzr_as_second_source() {
    let (mut cpu, mut bus) = setup();
    let addr = RAM_BASE + 0x1a00;
    cpu.regs.set_x(29, addr);
    cpu.regs.set_x(14, 0x1122_3344_5566_7788);
    cpu.reserve_exclusive(addr, 16);

    execute(&mut cpu, &mut bus, decode(0xC82E_7FAE).unwrap()).unwrap(); // stxp w14, x14, xzr, [x29]

    assert_eq!(bus.mem.read(addr, 8), Some(0x1122_3344_5566_7788));
    assert_eq!(bus.mem.read(addr + 8, 8), Some(0));
    assert_eq!(cpu.regs.x(14), 0);
}
