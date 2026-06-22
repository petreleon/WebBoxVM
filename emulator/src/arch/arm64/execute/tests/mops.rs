use super::*;

const CPYFP_X3_X2_X1: u32 = 0x1901_0443;
const SETP_X3_X2_X1: u32 = 0x19C1_0443;
const MOPS_VA_PAGE: u64 = 0x1000;
const MOPS_FIRST_PA: u64 = RAM_BASE + 0x0500_0000;
const MOPS_SECOND_PA: u64 = RAM_BASE + 0x0600_0000;

#[test]
fn mops_forward_copy_triplet_copies_bytes_and_consumes_size() {
    let (mut cpu, mut bus) = setup();
    for i in 0..8 {
        bus.mem.write(RAM_BASE + 0x100 + i, 1, 0xA0 + i);
    }
    cpu.regs.set_x(1, RAM_BASE + 0x100);
    cpu.regs.set_x(2, 8);
    cpu.regs.set_x(3, RAM_BASE + 0x200);

    for raw in [0x1901_0443, 0x1941_0443, 0x1981_0443] {
        execute(&mut cpu, &mut bus, decode(raw).unwrap()).unwrap();
    }

    for i in 0..8 {
        assert_eq!(bus.mem.read(RAM_BASE + 0x200 + i, 1), Some(0xA0 + i));
    }
    assert_eq!(cpu.regs.x(1), RAM_BASE + 0x108);
    assert_eq!(cpu.regs.x(2), 0);
    assert_eq!(cpu.regs.x(3), RAM_BASE + 0x208);
    assert!(cpu.pstate.c());
}

#[test]
fn mops_overlap_copy_uses_backward_direction() {
    let (mut cpu, mut bus) = setup();
    for i in 0..8 {
        bus.mem.write(RAM_BASE + i, 1, i);
    }
    cpu.regs.set_x(1, RAM_BASE);
    cpu.regs.set_x(2, 8);
    cpu.regs.set_x(3, RAM_BASE + 2);

    execute(&mut cpu, &mut bus, decode(0x1D01_0443).unwrap()).unwrap();

    let copied = [0, 1, 2, 3, 4, 5, 6, 7];
    for (i, expected) in copied.into_iter().enumerate() {
        assert_eq!(bus.mem.read(RAM_BASE + 2 + i as u64, 1), Some(expected));
    }
    assert!(cpu.pstate.n());
    assert!(cpu.pstate.c());
}

#[test]
fn mops_copy_same_page_bulk_clears_exclusive_across_full_range() {
    let (mut cpu, mut bus) = setup();
    let src = RAM_BASE + 0x8000;
    let dst = RAM_BASE + 0x9000;
    for offset in 0..300u64 {
        bus.mem.write(src + offset, 1, offset);
    }
    cpu.regs.set_x(1, src);
    cpu.regs.set_x(2, 300);
    cpu.regs.set_x(3, dst);
    cpu.reserve_exclusive(dst + 260, 8);

    execute(&mut cpu, &mut bus, decode(CPYFP_X3_X2_X1).unwrap()).unwrap();

    assert_eq!(bus.mem.read(dst, 1), Some(0));
    assert_eq!(bus.mem.read(dst + 260, 1), Some(260 & 0xff));
    assert_eq!(bus.mem.read(dst + 299, 1), Some(299 & 0xff));
    assert_eq!(cpu.regs.x(1), src + 300);
    assert_eq!(cpu.regs.x(2), 0);
    assert_eq!(cpu.regs.x(3), dst + 300);
    assert!(cpu.exclusive.is_none());
}

#[test]
fn mops_copy_cross_page_fault_preserves_partial_write_order() {
    let (mut cpu, mut bus) = setup();
    let src = MOPS_VA_PAGE + 0x100;
    let dst = MOPS_VA_PAGE + PAGE_SIZE - 2;
    map_two_user_pages(
        &mut cpu,
        &mut bus,
        MOPS_VA_PAGE,
        MOPS_FIRST_PA,
        MOPS_SECOND_PA,
    );
    for offset in 0..4u64 {
        bus.mem
            .write(MOPS_FIRST_PA + 0x100 + offset, 1, 0xe0 + offset);
        bus.mem.write(MOPS_SECOND_PA + offset, 1, 0xaa);
    }
    unmap_second_mops_page(&mut bus);
    cpu.regs.set_x(1, src);
    cpu.regs.set_x(2, 4);
    cpu.regs.set_x(3, dst);

    let err = execute(&mut cpu, &mut bus, decode(CPYFP_X3_X2_X1).unwrap()).unwrap_err();

    assert_eq!(err, "translation fault");
    assert_eq!(bus.mem.read(MOPS_FIRST_PA + PAGE_SIZE - 2, 1), Some(0xe0));
    assert_eq!(bus.mem.read(MOPS_FIRST_PA + PAGE_SIZE - 1, 1), Some(0xe1));
    assert_eq!(bus.mem.read(MOPS_SECOND_PA, 1), Some(0xaa));
    assert_eq!(cpu.regs.x(1), src);
    assert_eq!(cpu.regs.x(2), 4);
    assert_eq!(cpu.regs.x(3), dst);
}

#[test]
fn mops_set_same_page_bulk_clears_exclusive_across_full_range() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x6000;
    cpu.regs.set_x(1, 0x5a);
    cpu.regs.set_x(2, 300);
    cpu.regs.set_x(3, base);
    cpu.reserve_exclusive(base + 260, 8);

    execute(&mut cpu, &mut bus, decode(SETP_X3_X2_X1).unwrap()).unwrap();

    assert_eq!(bus.mem.read(base, 1), Some(0x5a));
    assert_eq!(bus.mem.read(base + 260, 1), Some(0x5a));
    assert_eq!(bus.mem.read(base + 299, 1), Some(0x5a));
    assert_eq!(cpu.regs.x(2), 0);
    assert_eq!(cpu.regs.x(3), base + 300);
    assert!(cpu.exclusive.is_none());
}

#[test]
fn mops_set_cross_page_fault_preserves_partial_write_order() {
    let (mut cpu, mut bus) = setup();
    let va = MOPS_VA_PAGE + PAGE_SIZE - 2;
    map_two_user_pages(
        &mut cpu,
        &mut bus,
        MOPS_VA_PAGE,
        MOPS_FIRST_PA,
        MOPS_SECOND_PA,
    );
    unmap_second_mops_page(&mut bus);
    cpu.regs.set_x(1, 0xcd);
    cpu.regs.set_x(2, 4);
    cpu.regs.set_x(3, va);
    for offset in 0..4u64 {
        bus.mem.write(MOPS_SECOND_PA + offset, 1, 0xaa);
    }

    let err = execute(&mut cpu, &mut bus, decode(SETP_X3_X2_X1).unwrap()).unwrap_err();

    assert_eq!(err, "translation fault");
    assert_eq!(bus.mem.read(MOPS_FIRST_PA + PAGE_SIZE - 2, 1), Some(0xcd));
    assert_eq!(bus.mem.read(MOPS_FIRST_PA + PAGE_SIZE - 1, 1), Some(0xcd));
    assert_eq!(bus.mem.read(MOPS_SECOND_PA, 1), Some(0xaa));
    assert_eq!(cpu.regs.x(2), 4);
    assert_eq!(cpu.regs.x(3), va);
}

#[test]
fn mops_set_triplet_writes_low_source_byte() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0xABCD);
    cpu.regs.set_x(2, 6);
    cpu.regs.set_x(3, RAM_BASE + 0x300);

    for raw in [0x19C1_0443, 0x19C1_4443, 0x19C1_8443] {
        execute(&mut cpu, &mut bus, decode(raw).unwrap()).unwrap();
    }

    for i in 0..6 {
        assert_eq!(bus.mem.read(RAM_BASE + 0x300 + i, 1), Some(0xCD));
    }
    assert_eq!(cpu.regs.x(2), 0);
    assert_eq!(cpu.regs.x(3), RAM_BASE + 0x306);
    assert!(cpu.pstate.c());
}

fn unmap_second_mops_page(bus: &mut SystemBus) {
    let l3 = RAM_BASE + 2 * PAGE_SIZE;
    let l3_idx = (MOPS_VA_PAGE >> PT_L3_SHIFT) & 0x1ff;
    bus.mem.write(l3 + (l3_idx + 1) * 8, 8, 0);
}
