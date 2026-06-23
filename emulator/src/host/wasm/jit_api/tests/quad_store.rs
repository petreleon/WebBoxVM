use super::super::quad_store::stage_jit_quad_store_from_machine;
use super::super::store::apply_jit_pending_stores;
use super::map_two_ttbr0_pages;
use crate::constants::{PAGE_SIZE, RAM_BASE};
use crate::runtime::Machine;

#[test]
fn jit_quad_store_stages_four_values_before_commit() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    let base = RAM_BASE + 0x4000;

    stage_jit_quad_store_from_machine(&mut machine, 0, base, 8, [1, 2, 3, 4], &mut stores)
        .expect("quad store should stage RAM writes");

    assert_eq!(stores.len(), 4);
    assert_eq!(machine.bus.mem.read(base, 8), Some(0));
    apply_jit_pending_stores(&mut machine, &stores).expect("apply staged quad stores");
    for index in 0..4 {
        assert_eq!(machine.bus.mem.read(base + index * 8, 8), Some(index + 1));
    }
}

#[test]
fn jit_quad_store_falls_back_across_noncontiguous_pages() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    map_two_ttbr0_pages(&mut machine, RAM_BASE + 0x3000, RAM_BASE + 0x8000);

    stage_jit_quad_store_from_machine(
        &mut machine,
        0,
        PAGE_SIZE - 16,
        8,
        [1, 2, 3, 4],
        &mut stores,
    )
    .expect("cross-page quad store should translate both pages");

    apply_jit_pending_stores(&mut machine, &stores).expect("apply staged quad stores");
    assert_eq!(machine.bus.mem.read(RAM_BASE + 0x3ff0, 8), Some(1));
    assert_eq!(machine.bus.mem.read(RAM_BASE + 0x3ff8, 8), Some(2));
    assert_eq!(machine.bus.mem.read(RAM_BASE + 0x8000, 8), Some(3));
    assert_eq!(machine.bus.mem.read(RAM_BASE + 0x8008, 8), Some(4));
}

#[test]
fn jit_quad_store_keeps_staging_atomic_when_second_pair_faults() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    map_two_ttbr0_pages(&mut machine, RAM_BASE + 0x3000, RAM_BASE + 0x8000);
    machine.bus.mem.write(RAM_BASE + 0x2008, 8, 0);

    let err = stage_jit_quad_store_from_machine(
        &mut machine,
        0,
        PAGE_SIZE - 16,
        8,
        [1, 2, 3, 4],
        &mut stores,
    )
    .expect_err("invalid second page should fault the quad store");

    assert!(err.contains("JIT store helper"), "{err}");
    assert!(stores.is_empty());
}
