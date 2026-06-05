use super::*;

#[test]
fn sve_st1h_scalar_base_forms_store_low_halfwords() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;
    let base = RAM_BASE + 0xB000;
    cpu.regs.set_x(5, base);

    cpu.sve_pred[3][0] = (1 << 0) | (1 << 2);
    cpu.simd[7] = 0xABCD_5678_1234;
    for offset in 0..8 {
        bus.write(base + offset, 1, 0xCC);
    }
    execute(&mut cpu, &mut bus, decode(0xE4A0_ECA7).unwrap()).unwrap();
    assert_eq!(bus.mem.read(base, 2), Some(0x1234));
    assert_eq!(bus.mem.read(base + 2, 2), Some(0x5678));
    assert_eq!(bus.mem.read(base + 4, 2), Some(0xCCCC));

    cpu.sve_pred[3][0] = 1;
    cpu.simd[7] = 0xCAFE;
    execute(&mut cpu, &mut bus, decode(0xE4C1_ECA7).unwrap()).unwrap();
    assert_eq!(bus.mem.read(base + 8, 2), Some(0xCAFE));
}

#[test]
fn sve_st1h_register_offset_scales_scalar_index_by_halfword() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 32;
    let base = RAM_BASE + 0xC000;
    cpu.regs.set_x(0, base);
    cpu.regs.set_x(30, 4);

    cpu.sve_pred[7][0] = 1;
    cpu.simd[31] = 0xDEAD_BEEF;
    for offset in 0..12 {
        bus.write(base + offset, 1, 0xAA);
    }
    execute(&mut cpu, &mut bus, decode(0xE4FE_5C1F).unwrap()).unwrap();
    assert_eq!(bus.mem.read(base + 8, 2), Some(0xBEEF));
    assert_eq!(bus.mem.read(base + 10, 2), Some(0xAAAA));
}
