use super::super::quad_load::jit_load_quad_guest_from_machine;
use super::map_two_ttbr0_pages;
use crate::constants::{PAGE_SIZE, RAM_BASE};
use crate::host::wasm::JitPendingStore;
use crate::runtime::Machine;

#[test]
fn jit_quad_load_reads_adjacent_mapped_values() {
    let mut machine = Machine::new(1);
    map_two_ttbr0_pages(&mut machine, RAM_BASE + 0x3000, RAM_BASE + 0x8000);
    for index in 0..4 {
        machine
            .bus
            .mem
            .write(RAM_BASE + 0x3010 + index * 8, 8, 0x1000 + index as u64);
    }

    let values = jit_load_quad_guest_from_machine(&mut machine, 0, 0x10, 8, &[])
        .expect("JIT quad load helper should read RAM");

    assert_eq!(values, [0x1000, 0x1001, 0x1002, 0x1003]);
}

#[test]
fn jit_quad_load_forwards_pending_store_bytes() {
    let mut machine = Machine::new(1);
    machine
        .bus
        .mem
        .write(RAM_BASE + 0x40, 8, 0x8877_6655_4433_2211);
    let bytes = 0xaabb_ccdd_u32.to_le_bytes();
    let stores = [JitPendingStore::new(RAM_BASE + 0x44, &bytes)];

    let values = jit_load_quad_guest_from_machine(&mut machine, 0, RAM_BASE + 0x40, 8, &stores)
        .expect("JIT quad load helper should forward staged bytes");

    assert_eq!(values[0], 0xaabb_ccdd_4433_2211);
    assert_eq!(machine.bus.mem.read(RAM_BASE + 0x44, 4), Some(0x8877_6655));
}

#[test]
fn jit_quad_load_ignores_disjoint_pending_store() {
    let mut machine = Machine::new(1);
    for index in 0..4 {
        machine
            .bus
            .mem
            .write(RAM_BASE + 0x40 + index * 8, 8, 0x1000 + index as u64);
    }
    let bytes = 0xaabb_ccdd_u32.to_le_bytes();
    let stores = [JitPendingStore::new(RAM_BASE + 0x90, &bytes)];

    let values = jit_load_quad_guest_from_machine(&mut machine, 0, RAM_BASE + 0x40, 8, &stores)
        .expect("disjoint staged store should not affect quad load");

    assert_eq!(values, [0x1000, 0x1001, 0x1002, 0x1003]);
}

#[test]
fn jit_quad_load_falls_back_across_noncontiguous_pages() {
    let mut machine = Machine::new(1);
    map_two_ttbr0_pages(&mut machine, RAM_BASE + 0x3000, RAM_BASE + 0x8000);
    for (pa, value) in [
        (RAM_BASE + 0x3ff0, 1),
        (RAM_BASE + 0x3ff8, 2),
        (RAM_BASE + 0x8000, 3),
        (RAM_BASE + 0x8008, 4),
    ] {
        machine.bus.mem.write(pa, 8, value);
    }

    let values = jit_load_quad_guest_from_machine(&mut machine, 0, PAGE_SIZE - 16, 8, &[])
        .expect("cross-page quad load should translate both pages");

    assert_eq!(values, [1, 2, 3, 4]);
}

#[test]
fn jit_quad_load_rejects_non_lane_widths() {
    let mut machine = Machine::new(1);

    let err = jit_load_quad_guest_from_machine(&mut machine, 0, RAM_BASE, 4, &[])
        .expect_err("quad helper must reject non-64-bit lanes");

    assert!(err.contains("quad load size 4"), "{err}");
}
