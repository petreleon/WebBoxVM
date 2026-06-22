use super::*;
use crate::arch::arm64::jit::hash_raw_words;
use crate::host::wasm::jit_api::validate::{code_page_generations, validate_jit_block};

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
    let (start_generation, end_generation) =
        code_page_generations(&machine.bus.mem, start_pa, 2).expect("code page generations");

    let err = validate_jit_block(
        &machine,
        0,
        start_pc,
        start_pa,
        hash,
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
    let (start_generation, end_generation) =
        code_page_generations(&machine.bus.mem, start_pa, 2).expect("code page generations");

    machine.bus.mem.write(start_pa + 4, 4, 0xd420_0000);

    let err = validate_jit_block(
        &machine,
        0,
        start_pc,
        start_pa,
        hash,
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

    validate_jit_block(
        &machine,
        0,
        start_pc,
        start_pa,
        0,
        start_generation,
        end_generation,
        1,
    )
    .expect("matching page generations skip raw hash validation");
}
