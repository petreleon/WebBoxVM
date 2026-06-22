use super::load::jit_load_guest_from_machine;
use super::store::{apply_jit_pending_stores, stage_jit_store_from_machine};
use super::validate::validate_jit_block;
use crate::arch::arm64::jit::WasmJitCpuState;
use crate::arch::arm64::jit::hash_raw_words;
use crate::constants::{
    DESC_AF_BIT, DESC_BLOCK, DESC_TABLE, PL011_UART_IRQ_ID, RAM_BASE, SCTLR_MMU_ENABLE,
    TCR_T1SZ_SHIFT, UART_BASE, UART_IMSC_OFFSET,
};
use crate::runtime::Machine;

use super::commit::commit_jit_state;

mod exclusive;
mod side_effects;
mod sysreg;
mod timer;
mod validate;

const NOP: u32 = 0xd503_201f;
const UART_RX_IRQ_MASK: u64 = (1 << 4) | (1 << 6);

fn map_two_ttbr0_pages(machine: &mut Machine, page0_pa: u64, page1_pa: u64) {
    let l1_table = RAM_BASE;
    let l2_table = RAM_BASE + 0x1000;
    let l3_table = RAM_BASE + 0x2000;

    machine.bus.mem.write(l1_table, 8, l2_table | DESC_TABLE);
    machine.bus.mem.write(l2_table, 8, l3_table | DESC_TABLE);
    machine
        .bus
        .mem
        .write(l3_table, 8, page0_pa | DESC_AF_BIT | DESC_BLOCK);
    machine
        .bus
        .mem
        .write(l3_table + 8, 8, page1_pa | DESC_AF_BIT | DESC_BLOCK);

    let cpu = &mut machine.cpus[0];
    cpu.sys.ttbr0_el1 = l1_table;
    cpu.sys.tcr_el1 = (25 << TCR_T1SZ_SHIFT) | 25;
    cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE;
}

#[test]
fn validate_jit_block_rejects_changed_second_instruction_translation() {
    let mut machine = Machine::new(1);
    let start_pc = 0xffc;
    let start_pa = RAM_BASE + 0x3ffc;
    map_two_ttbr0_pages(&mut machine, RAM_BASE + 0x3000, RAM_BASE + 0x8000);
    machine.cpus[0].regs.pc = start_pc;
    machine.bus.mem.write(start_pa, 4, NOP as u64);
    machine.bus.mem.write(start_pa + 4, 4, NOP as u64);

    let hash = hash_raw_words(start_pa, [NOP, NOP]);
    let err = validate_jit_block(&machine, 0, start_pc, start_pa, hash, 2)
        .expect_err("non-contiguous second instruction mapping must be rejected");

    assert!(
        err.contains("cached JIT block PA changed at PC 0x0000000000001000"),
        "{err}"
    );
}

#[test]
fn commit_rejects_jit_block_when_uart_input_still_asserts_irq() {
    let mut machine = Machine::new(1);
    machine.bus.gic.enable_interrupt(PL011_UART_IRQ_ID);
    machine
        .bus
        .uart
        .write(UART_BASE + UART_IMSC_OFFSET, 4, UART_RX_IRQ_MASK);
    machine.bus.uart.feed_input("ab");
    machine.bus.gic.clear_pending(PL011_UART_IRQ_ID);
    machine.cpus[0].pstate = machine.cpus[0].pstate.with_irq_masked(false);
    let state = WasmJitCpuState::from_cpu(&machine.cpus[0]);

    let err = commit_jit_state(&state, &mut machine, 0, 1, state.pc)
        .expect_err("queued UART input must block JIT commit");

    assert!(
        err.contains("pending IRQ boundary"),
        "unexpected JIT commit error: {err}"
    );
}

#[test]
fn jit_load_guest_reads_mapped_ram() {
    let mut machine = Machine::new(1);
    map_two_ttbr0_pages(&mut machine, RAM_BASE + 0x3000, RAM_BASE + 0x8000);
    machine.bus.mem.write(RAM_BASE + 0x3010, 4, 0x4433_2211);

    let value = jit_load_guest_from_machine(&mut machine, 0, 0x10, 4, &[])
        .expect("JIT load helper should read RAM");

    assert_eq!(value, 0x4433_2211);
}

#[test]
fn jit_load_guest_rejects_device_reads() {
    let mut machine = Machine::new(1);

    let err = jit_load_guest_from_machine(&mut machine, 0, UART_BASE, 4, &[])
        .expect_err("JIT load helper must reject MMIO");

    assert!(err.contains("device PA"), "{err}");
}

#[test]
fn jit_store_guest_stages_until_applied() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    machine.bus.mem.write(RAM_BASE + 0x40, 4, 0);

    stage_jit_store_from_machine(
        &mut machine,
        0,
        RAM_BASE + 0x40,
        4,
        0x4433_2211,
        &mut stores,
    )
    .expect("JIT store helper should stage RAM write");

    assert_eq!(machine.bus.mem.read(RAM_BASE + 0x40, 4), Some(0));
    apply_jit_pending_stores(&mut machine, &stores).expect("apply staged store");
    assert_eq!(machine.bus.mem.read(RAM_BASE + 0x40, 4), Some(0x4433_2211));
}

#[test]
fn jit_store_commit_updates_exclusive_reservations() {
    for (store_pa, should_match) in [(RAM_BASE + 0x42, false), (RAM_BASE + 0x80, true)] {
        let mut machine = Machine::new(1);
        let mut stores = Vec::new();
        machine.cpus[0].reserve_exclusive(RAM_BASE + 0x40, 8);
        machine.bus.mem.write(store_pa, 4, 0);

        stage_jit_store_from_machine(&mut machine, 0, store_pa, 4, 1, &mut stores)
            .expect("stage RAM store");
        apply_jit_pending_stores(&mut machine, &stores).expect("apply staged store");

        assert_eq!(
            machine.cpus[0].exclusive_matches(RAM_BASE + 0x40, 8),
            should_match
        );
    }
}

#[test]
fn jit_load_guest_forwards_pending_store_bytes() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    machine
        .bus
        .mem
        .write(RAM_BASE + 0x40, 8, 0x8877_6655_4433_2211);

    stage_jit_store_from_machine(&mut machine, 0, RAM_BASE + 0x42, 2, 0xaabb, &mut stores)
        .expect("stage overlapping store");

    let value = jit_load_guest_from_machine(&mut machine, 0, RAM_BASE + 0x40, 4, &stores)
        .expect("JIT load helper should forward staged bytes");

    assert_eq!(value, 0xaabb_2211);
    assert_eq!(machine.bus.mem.read(RAM_BASE + 0x40, 4), Some(0x4433_2211));
}

#[test]
fn jit_store_guest_rejects_device_writes() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();

    let err = stage_jit_store_from_machine(&mut machine, 0, UART_BASE, 4, 1, &mut stores)
        .expect_err("JIT store helper must reject MMIO");

    assert!(err.contains("rejected PA"), "{err}");
    assert!(stores.is_empty());
}
