use super::*;

#[test]
fn ldp_loads_pair() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0x4000_0000);
    bus.mem.write(0x4000_0000, 8, 0xDEAD_BEEF);
    bus.mem.write(0x4000_0008, 8, 0xCAFE_BABE);
    execute(&mut cpu, &mut bus, decode(0xA940_0C22).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(2), 0xDEAD_BEEF);
    assert_eq!(cpu.regs.x(3), 0xCAFE_BABE);
}

#[test]
fn scalar_load_store_translate_each_page_when_crossing_boundary() {
    let (mut cpu, mut bus) = setup();
    let va = 0x1ffc;
    let first_pa = RAM_BASE + 0x0100_0000;
    let second_pa = RAM_BASE + 0x0200_0000;
    map_two_user_pages(&mut cpu, &mut bus, 0x1000, first_pa, second_pa);

    bus.mem.write(first_pa + 0xffc, 4, 0x5566_7788);
    bus.mem.write(second_pa, 4, 0x1122_3344);
    cpu.regs.set_x(1, va);
    execute(&mut cpu, &mut bus, decode(0xF940_0022).unwrap()).unwrap(); // ldr x2, [x1]
    assert_eq!(cpu.regs.x(2), 0x1122_3344_5566_7788);

    bus.mem.write(first_pa + PAGE_SIZE, 4, 0xDEAD_BEEF);
    cpu.regs.set_x(0, 0xAABB_CCDD_EEFF_0011);
    execute(&mut cpu, &mut bus, decode(0xF900_0020).unwrap()).unwrap(); // str x0, [x1]

    assert_eq!(bus.mem.read(first_pa + 0xffc, 4), Some(0xEEFF_0011));
    assert_eq!(bus.mem.read(second_pa, 4), Some(0xAABB_CCDD));
    assert_eq!(bus.mem.read(first_pa + PAGE_SIZE, 4), Some(0xDEAD_BEEF));
}

#[test]
fn ldpsw_loads_and_sign_extends_pair() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(19, 0x4000_0000);
    bus.mem.write(0x4000_0064, 4, 0xffff_fffc);
    bus.mem.write(0x4000_0068, 4, 0x7fff_fffe);

    execute(&mut cpu, &mut bus, decode(0x694C_9262).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(2), 0xffff_ffff_ffff_fffc);
    assert_eq!(cpu.regs.x(4), 0x7fff_fffe);
}
