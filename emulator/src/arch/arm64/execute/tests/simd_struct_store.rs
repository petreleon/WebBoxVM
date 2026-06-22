use super::simd_helpers::*;
use super::*;

const ST2_V30_V31_16B_X1: u32 = 0x4C00_803E;
const CROSS_PAGE_VA: u64 = 0x1000;
const CROSS_PAGE_FIRST_PA: u64 = RAM_BASE + 0x0300_0000;
const CROSS_PAGE_SECOND_PA: u64 = RAM_BASE + 0x0400_0000;

#[test]
fn simd_structured_stores_interleave_vector_lanes() {
    let (mut cpu, mut bus) = setup();
    let st2_base = RAM_BASE + 0x4800;
    cpu.regs.set_x(1, st2_base);
    cpu.simd[30] = vector_bytes(0x00);
    cpu.simd[31] = vector_bytes(0x40);

    execute(&mut cpu, &mut bus, decode(0x4C00_803E).unwrap()).unwrap();

    for lane in 0..16u64 {
        assert_eq!(bus.read(st2_base + lane * 2, 1), Some(lane));
        assert_eq!(bus.read(st2_base + lane * 2 + 1, 1), Some(0x40 + lane));
    }
    assert_eq!(cpu.regs.x(1), st2_base);
}

#[test]
fn simd_structured_stores_apply_element_size_and_post_index() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x4c00;
    cpu.regs.set_x(1, base);
    for reg in 0..4 {
        cpu.simd[22 + reg] = vector_bytes((reg as u64 + 1) * 0x20);
    }

    execute(&mut cpu, &mut bus, decode(0x4C9F_0436).unwrap()).unwrap();

    for lane in 0..8u64 {
        for reg in 0..4u64 {
            for byte in 0..2u64 {
                let offset = (lane * 4 + reg) * 2 + byte;
                let value = (reg + 1) * 0x20 + lane * 2 + byte;
                assert_eq!(bus.read(base + offset, 1), Some(value));
            }
        }
    }
    assert_eq!(cpu.regs.x(1), base + 64);
}

#[test]
fn simd_structured_same_page_store_clears_overlapping_exclusive() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x5800;
    cpu.regs.set_x(1, base);
    cpu.simd[30] = vector_bytes(0);
    cpu.simd[31] = vector_bytes(0x40);
    cpu.reserve_exclusive(base + 17, 4);

    execute(&mut cpu, &mut bus, decode(ST2_V30_V31_16B_X1).unwrap()).unwrap();

    assert_eq!(bus.read(base + 16, 1), Some(8));
    assert_eq!(bus.read(base + 17, 1), Some(0x48));
    assert!(cpu.exclusive.is_none());
}

#[test]
fn simd_structured_cross_page_store_preserves_partial_fault_order() {
    let (mut cpu, mut bus) = setup();
    let va = CROSS_PAGE_VA + PAGE_SIZE - 16;
    map_two_user_pages(
        &mut cpu,
        &mut bus,
        CROSS_PAGE_VA,
        CROSS_PAGE_FIRST_PA,
        CROSS_PAGE_SECOND_PA,
    );
    unmap_second_cross_page(&mut bus);
    cpu.regs.set_x(1, va);
    cpu.simd[30] = vector_bytes(0);
    cpu.simd[31] = vector_bytes(0x40);
    for offset in 0..16u64 {
        bus.mem.write(CROSS_PAGE_SECOND_PA + offset, 1, 0xaa);
    }

    let err = execute(&mut cpu, &mut bus, decode(ST2_V30_V31_16B_X1).unwrap()).unwrap_err();

    assert_eq!(err, "translation fault");
    for offset in 0..16u64 {
        let lane = offset / 2;
        let expected = if offset % 2 == 0 { lane } else { 0x40 + lane };
        assert_eq!(
            bus.mem
                .read(CROSS_PAGE_FIRST_PA + PAGE_SIZE - 16 + offset, 1),
            Some(expected)
        );
        assert_eq!(bus.mem.read(CROSS_PAGE_SECOND_PA + offset, 1), Some(0xaa));
    }
}

#[test]
fn simd_st4_single_lane_stores_one_element_from_four_registers() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x5000;
    cpu.regs.set_x(0, base);
    cpu.simd[19] = 0xaaaau128 << 32 | 0x1122_3344;
    cpu.simd[20] = 0xbbbbu128 << 32 | 0x5566_7788;
    cpu.simd[21] = 0xccccu128 << 32 | 0x99aa_bbcc;
    cpu.simd[22] = 0xddddu128 << 32 | 0xddee_ff00;

    execute(&mut cpu, &mut bus, decode(0x0DBF_A013).unwrap()).unwrap();

    assert_eq!(bus.read(base, 4), Some(0x1122_3344));
    assert_eq!(bus.read(base + 4, 4), Some(0x5566_7788));
    assert_eq!(bus.read(base + 8, 4), Some(0x99aa_bbcc));
    assert_eq!(bus.read(base + 12, 4), Some(0xddee_ff00));
    assert_eq!(cpu.regs.x(0), base + 16);
}

fn unmap_second_cross_page(bus: &mut SystemBus) {
    let l3 = RAM_BASE + 2 * PAGE_SIZE;
    let l3_idx = (CROSS_PAGE_VA >> PT_L3_SHIFT) & 0x1ff;
    bus.mem.write(l3 + (l3_idx + 1) * 8, 8, 0);
}
