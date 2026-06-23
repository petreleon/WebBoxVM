use super::super::exclusive_load::{
    apply_jit_pending_exclusive_reservation, jit_load_exclusive_pair_from_machine,
};
use super::map_two_ttbr0_pages;
use crate::constants::{PAGE_SIZE, RAM_BASE, UART_BASE};
use crate::runtime::Machine;

#[test]
fn jit_load_exclusive_pair_stages_combined_reservation() {
    let mut machine = Machine::new(1);
    let addr = RAM_BASE + 0x600;
    machine.bus.mem.write(addr, 8, 0x1122_3344_5566_7788);
    machine.bus.mem.write(addr + 8, 8, 0x99aa_bbcc_ddee_ff00);

    let (value1, value2, reservation) =
        jit_load_exclusive_pair_from_machine(&mut machine, 0, addr, 8, &[])
            .expect("exclusive pair load helper should read RAM");

    assert_eq!(value1, 0x1122_3344_5566_7788);
    assert_eq!(value2, 0x99aa_bbcc_ddee_ff00);
    assert!(machine.cpus[0].exclusive.is_none());
    apply_jit_pending_exclusive_reservation(&mut machine, Some(reservation));
    assert!(machine.cpus[0].exclusive_matches(addr, 16));
}

#[test]
fn jit_load_exclusive_pair_translates_split_pages() {
    let mut machine = Machine::new(1);
    let first_pa = RAM_BASE + 0x3000;
    let second_pa = RAM_BASE + 0x8000;
    map_two_ttbr0_pages(&mut machine, first_pa, second_pa);
    machine.bus.mem.write(first_pa + 0xffc, 4, 0x1122_3344);
    machine.bus.mem.write(second_pa, 4, 0x5566_7788);

    let (value1, value2, reservation) =
        jit_load_exclusive_pair_from_machine(&mut machine, 0, PAGE_SIZE - 4, 4, &[])
            .expect("cross-page exclusive pair load should translate both pages");

    assert_eq!(value1, 0x1122_3344);
    assert_eq!(value2, 0x5566_7788);
    assert_eq!(reservation.pa, first_pa + 0xffc);
    assert_eq!(reservation.size, 8);
}

#[test]
fn jit_load_exclusive_pair_rejects_device_reads() {
    let mut machine = Machine::new(1);

    let err = jit_load_exclusive_pair_from_machine(&mut machine, 0, UART_BASE, 4, &[])
        .expect_err("exclusive pair load helper must reject MMIO");

    assert!(err.contains("device PA"), "{err}");
}
