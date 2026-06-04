use super::validate::validate_jit_block;
use crate::arm64::jit::hash_raw_words;
use crate::arm64::machine::Machine;
use crate::constants::{
    DESC_AF_BIT, DESC_BLOCK, DESC_TABLE, RAM_BASE, SCTLR_MMU_ENABLE, TCR_T1SZ_SHIFT,
};

const NOP: u32 = 0xd503_201f;

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
