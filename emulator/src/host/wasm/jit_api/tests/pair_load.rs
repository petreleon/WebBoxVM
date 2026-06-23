use super::super::pair_load::jit_load_pair_guest_from_machine;
use super::map_two_ttbr0_pages;
use crate::constants::{PAGE_SIZE, RAM_BASE};
use crate::host::wasm::JitPendingStore;
use crate::runtime::Machine;

#[test]
fn jit_pair_load_reads_adjacent_mapped_values() {
    let mut machine = Machine::new(1);
    map_two_ttbr0_pages(&mut machine, RAM_BASE + 0x3000, RAM_BASE + 0x8000);
    machine
        .bus
        .mem
        .write(RAM_BASE + 0x3010, 8, 0x8877_6655_4433_2211);
    machine
        .bus
        .mem
        .write(RAM_BASE + 0x3018, 8, 0x00ff_eedd_ccbb_aa99);

    let values = jit_load_pair_guest_from_machine(&mut machine, 0, 0x10, 8, &[])
        .expect("JIT pair load helper should read RAM");

    assert_eq!(values, (0x8877_6655_4433_2211, 0x00ff_eedd_ccbb_aa99));
}

#[test]
fn jit_pair_load_forwards_pending_store_bytes() {
    let mut machine = Machine::new(1);
    machine
        .bus
        .mem
        .write(RAM_BASE + 0x40, 8, 0x8877_6655_4433_2211);
    machine.bus.mem.write(RAM_BASE + 0x44, 4, 0x0102_0304);
    let bytes = 0xaabb_ccdd_u32.to_le_bytes();
    let stores = [JitPendingStore::new(RAM_BASE + 0x44, &bytes)];

    let values = jit_load_pair_guest_from_machine(&mut machine, 0, RAM_BASE + 0x40, 4, &stores)
        .expect("JIT pair load helper should forward staged bytes");

    assert_eq!(values, (0x4433_2211, 0xaabb_ccdd));
    assert_eq!(machine.bus.mem.read(RAM_BASE + 0x44, 4), Some(0x0102_0304));
}

#[test]
fn jit_pair_load_ignores_disjoint_pending_store() {
    let mut machine = Machine::new(1);
    machine
        .bus
        .mem
        .write(RAM_BASE + 0x40, 8, 0x1122_3344_5566_7788);
    machine
        .bus
        .mem
        .write(RAM_BASE + 0x48, 8, 0x99aa_bbcc_ddee_ff00);
    let bytes = 0xaabb_ccdd_u32.to_le_bytes();
    let stores = [JitPendingStore::new(RAM_BASE + 0x80, &bytes)];

    let values = jit_load_pair_guest_from_machine(&mut machine, 0, RAM_BASE + 0x40, 8, &stores)
        .expect("disjoint staged store should not affect pair load");

    assert_eq!(values, (0x1122_3344_5566_7788, 0x99aa_bbcc_ddee_ff00));
}

#[test]
fn jit_pair_load_falls_back_across_noncontiguous_pages() {
    let mut machine = Machine::new(1);
    map_two_ttbr0_pages(&mut machine, RAM_BASE + 0x3000, RAM_BASE + 0x8000);
    machine.bus.mem.write(RAM_BASE + 0x3ffc, 4, 0x4433_2211);
    machine.bus.mem.write(RAM_BASE + 0x8000, 4, 0x8877_6655);

    let values = jit_load_pair_guest_from_machine(&mut machine, 0, PAGE_SIZE - 4, 4, &[])
        .expect("cross-page pair load should translate both pages");

    assert_eq!(values, (0x4433_2211, 0x8877_6655));
}

#[test]
fn jit_pair_load_rejects_non_pair_widths() {
    let mut machine = Machine::new(1);

    let err = jit_load_pair_guest_from_machine(&mut machine, 0, RAM_BASE, 2, &[])
        .expect_err("JIT pair load helper must reject narrow pairs");

    assert!(err.contains("pair load size 2"), "{err}");
}
