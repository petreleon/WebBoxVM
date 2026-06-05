use super::*;

#[test]
fn mte_irg_and_gmi_use_logical_tag_bits() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(0, 0x1234);
    cpu.regs.set_x(1, 0b0011);

    execute(&mut cpu, &mut bus, decode(0x9AC1_1000).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0x0200_0000_0000_1234);

    execute(&mut cpu, &mut bus, decode(0x9ADF_1401).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(1), 0b0100);
}

#[test]
fn mte_ldg_clears_tag_in_tagless_model() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(0, 0x0A00_0000_0000_2000);

    execute(&mut cpu, &mut bus, decode(0xD960_0000).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(0), 0x2000);
}

#[test]
fn mte_addg_and_subg_adjust_address_and_clear_tag_in_tagless_model() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0x0A00_0000_0000_1000);

    execute(&mut cpu, &mut bus, decode(0x91BF_0C20).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0x1000 + 1008);

    cpu.regs.sp = 0x0B00_0000_0000_2000;
    execute(&mut cpu, &mut bus, decode(0xD1BF_3FFF).unwrap()).unwrap();
    assert_eq!(cpu.regs.sp, 0x2000 - 1008);
}

#[test]
fn mte_tag_stores_preserve_data_and_zeroing_stores_clear_granules() {
    let (mut cpu, mut bus) = setup();
    bus.mem.write(RAM_BASE, 8, u64::MAX);
    bus.mem.write(RAM_BASE + 8, 8, u64::MAX);
    cpu.regs.set_x(0, RAM_BASE);

    execute(&mut cpu, &mut bus, decode(0xD920_1400).unwrap()).unwrap();
    assert_eq!(bus.mem.read(RAM_BASE, 8), Some(u64::MAX));
    assert_eq!(cpu.regs.x(0), RAM_BASE + 16);

    cpu.regs.set_x(2, RAM_BASE);
    bus.mem.write(RAM_BASE + 64, 8, u64::MAX);
    bus.mem.write(RAM_BASE + 72, 8, u64::MAX);
    execute(&mut cpu, &mut bus, decode(0xD960_4C40).unwrap()).unwrap();
    assert_eq!(bus.mem.read(RAM_BASE + 64, 8), Some(0));
    assert_eq!(bus.mem.read(RAM_BASE + 72, 8), Some(0));
    assert_eq!(cpu.regs.x(2), RAM_BASE + 64);

    bus.mem.write(RAM_BASE, 8, u64::MAX);
    bus.mem.write(RAM_BASE + 24, 8, u64::MAX);
    cpu.regs.set_x(0, RAM_BASE);
    execute(&mut cpu, &mut bus, decode(0xD9E0_2400).unwrap()).unwrap();
    assert_eq!(bus.mem.read(RAM_BASE, 8), Some(0));
    assert_eq!(bus.mem.read(RAM_BASE + 24, 8), Some(0));
    assert_eq!(cpu.regs.x(0), RAM_BASE + 32);
}

#[test]
fn mte_stgp_stores_pair_data_in_tagless_model() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, RAM_BASE);
    cpu.regs.set_x(0, 0x1111_2222_3333_4444);
    cpu.regs.set_x(1, 0x5555_6666_7777_8888);

    execute(&mut cpu, &mut bus, decode(0x6900_0440).unwrap()).unwrap();
    assert_eq!(bus.mem.read(RAM_BASE, 8), Some(0x1111_2222_3333_4444));
    assert_eq!(bus.mem.read(RAM_BASE + 8, 8), Some(0x5555_6666_7777_8888));

    cpu.regs.set_x(10, RAM_BASE);
    cpu.regs.set_x(8, 0xAAAA_BBBB_CCCC_DDDD);
    cpu.regs.set_x(9, 0xEEEE_FFFF_0000_1111);
    execute(&mut cpu, &mut bus, decode(0x6881_2548).unwrap()).unwrap();
    assert_eq!(bus.mem.read(RAM_BASE, 8), Some(0xAAAA_BBBB_CCCC_DDDD));
    assert_eq!(bus.mem.read(RAM_BASE + 8, 8), Some(0xEEEE_FFFF_0000_1111));
    assert_eq!(cpu.regs.x(10), RAM_BASE + 32);

    cpu.regs.set_x(2, RAM_BASE + 8);
    let err = execute(&mut cpu, &mut bus, decode(0x6900_0440).unwrap()).unwrap_err();
    assert_eq!(err, "MTE tag granule alignment fault");
}
