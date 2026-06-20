use super::*;

#[test]
fn compiles_rev_prefix() {
    let block = block(vec![
        instr(Opcode::Rev, 1, 2, 0, 0, true),
        instr(Opcode::Rev, 3, 4, 0, 0, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile rev");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHL));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_OR));
}
