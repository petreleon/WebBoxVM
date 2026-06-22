use super::*;

#[test]
fn validate_jit_block_rejects_changed_raw_word() {
    let mut machine = Machine::new(1);
    let start_pc = RAM_BASE;
    let start_pa = RAM_BASE;
    machine.cpus[0].regs.pc = start_pc;
    machine.bus.mem.write(start_pa, 4, NOP as u64);
    machine.bus.mem.write(start_pa + 4, 4, NOP as u64);
    let hash = hash_raw_words(start_pa, [NOP, NOP]);

    machine.bus.mem.write(start_pa + 4, 4, 0xd420_0000);

    let err = validate_jit_block(&machine, 0, start_pc, start_pa, hash, 2)
        .expect_err("changed cached code bytes must invalidate JIT block");

    assert!(err.contains("raw hash changed"), "{err}");
}
