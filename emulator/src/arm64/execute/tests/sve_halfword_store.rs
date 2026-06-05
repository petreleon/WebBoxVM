use super::simd_helpers::{i32x4, u32x4, u64x2};
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

#[test]
fn sve_st1h_scatter_uses_vector_offsets_and_low_halfwords() {
    let (mut cpu, mut bus) = setup();
    cpu.sve_vl_bytes = 16;
    let base = RAM_BASE + 0xD020;
    cpu.regs.set_x(21, base);

    cpu.sve_pred[2][0] = (1 << 0) | (1 << 8);
    cpu.simd[18] = u64x2([0xAAAA_1111, 0xBBBB_2222]);
    cpu.simd[31] = u64x2([2, (-2i32 as u32) as u64]);
    for offset in 0..12 {
        bus.write(base - 4 + offset, 1, 0xCC);
    }
    execute(&mut cpu, &mut bus, decode(0xE4BF_CAB2).unwrap()).unwrap();
    assert_eq!(bus.mem.read(base + 4, 2), Some(0x1111));
    assert_eq!(bus.mem.read(base - 4, 2), Some(0x2222));
    assert_eq!(bus.mem.read(base + 6, 2), Some(0xCCCC));

    cpu.sve_pred[2][0] = (1 << 0) | (1 << 4);
    cpu.simd[18] = u32x4([0xDDDD_3333, 0xEEEE_4444, 0, 0]);
    cpu.simd[31] = i32x4([1, 5, 9, 13]);
    execute(&mut cpu, &mut bus, decode(0xE4DF_8AB2).unwrap()).unwrap();
    assert_eq!(bus.mem.read(base + 1, 2), Some(0x3333));
    assert_eq!(bus.mem.read(base + 5, 2), Some(0x4444));
}
