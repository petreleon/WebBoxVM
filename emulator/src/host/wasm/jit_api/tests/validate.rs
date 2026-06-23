use super::*;
use crate::arch::arm64::jit::{MAX_BLOCK_INSTRUCTIONS, hash_raw_words};
use crate::host::wasm::jit_api::validate::{
    code_page_generations, crosses_translation_page, validate_jit_block,
};

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
    let memory_generation = machine.bus.mem.generation();
    let (start_generation, end_generation) =
        code_page_generations(&machine.bus.mem, start_pa, 2).expect("code page generations");

    let err = validate_jit_block(
        &machine,
        0,
        start_pc,
        start_pa,
        hash,
        memory_generation,
        start_generation,
        end_generation,
        2,
    )
    .expect_err("non-contiguous second instruction mapping must be rejected");

    assert!(
        err.contains("cached JIT block PA changed at PC 0x0000000000001000"),
        "{err}"
    );
}

#[test]
fn validate_jit_block_rejects_changed_raw_word() {
    let mut machine = Machine::new(1);
    let start_pc = RAM_BASE;
    let start_pa = RAM_BASE;
    machine.cpus[0].regs.pc = start_pc;
    machine.bus.mem.write(start_pa, 4, NOP as u64);
    machine.bus.mem.write(start_pa + 4, 4, NOP as u64);
    let hash = hash_raw_words(start_pa, [NOP, NOP]);
    let memory_generation = machine.bus.mem.generation();
    let (start_generation, end_generation) =
        code_page_generations(&machine.bus.mem, start_pa, 2).expect("code page generations");

    machine.bus.mem.write(start_pa + 4, 4, 0xd420_0000);

    let err = validate_jit_block(
        &machine,
        0,
        start_pc,
        start_pa,
        hash,
        memory_generation,
        start_generation,
        end_generation,
        2,
    )
    .expect_err("changed cached code bytes must invalidate JIT block");

    assert!(err.contains("raw hash changed"), "{err}");
}

#[test]
fn validate_jit_block_skips_raw_hash_for_unchanged_code_pages() {
    let mut machine = Machine::new(1);
    let start_pc = RAM_BASE;
    let start_pa = RAM_BASE;
    machine.cpus[0].regs.pc = start_pc;
    machine.bus.mem.write(start_pa, 4, NOP as u64);
    let (start_generation, end_generation) =
        code_page_generations(&machine.bus.mem, start_pa, 1).expect("code page generations");
    let memory_generation = machine.bus.mem.generation();

    validate_jit_block(
        &machine,
        0,
        start_pc,
        start_pa,
        0,
        memory_generation,
        start_generation,
        end_generation,
        1,
    )
    .expect("matching page generations skip raw hash validation");
}

#[test]
fn validate_jit_block_skips_page_generations_when_memory_is_unchanged() {
    let mut machine = Machine::new(1);
    let start_pc = RAM_BASE;
    let start_pa = RAM_BASE;
    machine.cpus[0].regs.pc = start_pc;
    machine.bus.mem.write(start_pa, 4, NOP as u64);
    let memory_generation = machine.bus.mem.generation();

    validate_jit_block(
        &machine,
        0,
        start_pc,
        start_pa,
        0,
        memory_generation,
        999,
        999,
        1,
    )
    .expect("unchanged memory generation should skip stale page generations and hash");
}

#[test]
fn code_page_generations_reuses_same_page_generation() {
    let mut machine = Machine::new(1);
    let start_pa = RAM_BASE + 0x100;
    machine.bus.mem.write(start_pa, 4, NOP as u64);
    let (start_generation, end_generation) =
        code_page_generations(&machine.bus.mem, start_pa, 2).expect("code page generations");

    assert_eq!(start_generation, end_generation);
}

#[test]
fn translation_page_check_only_triggers_across_pages() {
    assert!(!crosses_translation_page(
        RAM_BASE,
        RAM_BASE + 0x100,
        RAM_BASE + 4,
        RAM_BASE + 0x104
    ));
    assert!(crosses_translation_page(
        0xffc,
        RAM_BASE + 0x3ffc,
        0x1000,
        RAM_BASE + 0x4000
    ));
}

#[test]
fn validate_jit_block_rejects_oversized_span() {
    let mut machine = Machine::new(1);
    machine.cpus[0].regs.pc = RAM_BASE;

    let err = validate_jit_block(
        &machine,
        0,
        RAM_BASE,
        RAM_BASE,
        0,
        0,
        0,
        0,
        MAX_BLOCK_INSTRUCTIONS + 1,
    )
    .expect_err("oversized cached block must not use endpoint validation");

    assert!(err.contains("maximum validation span"), "{err}");
}
