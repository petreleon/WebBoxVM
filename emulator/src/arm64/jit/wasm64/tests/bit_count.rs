use super::*;

#[test]
fn compiles_clz_64_and_32_bit_forms() {
    let block = block(vec![
        instr(Opcode::Clz, 1, 2, 0, 0, true),
        instr(Opcode::Clz, 3, 4, 0, 0, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile clz");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_CLZ));
    assert!(module.bytes.contains(&opcodes::OP_I32_CLZ));
    assert!(module.bytes.contains(&opcodes::OP_I64_EXTEND_I32_U));
}
