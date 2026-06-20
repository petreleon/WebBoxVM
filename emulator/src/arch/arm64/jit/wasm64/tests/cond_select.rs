use super::*;

#[test]
fn compiles_conditional_select_family() {
    let block = block(vec![
        instr_cond(Opcode::Csel, 0b1000, 0, true),
        instr_cond(Opcode::Csinc, 0b1001, 0, false),
        instr_cond(Opcode::Csinv, 0b1010, 0, true),
        instr_cond(Opcode::Csneg, 0b1011, 0, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile cond select");

    assert_eq!(module.guest_instr_count, 4);
    assert!(module.bytes.contains(&opcodes::OP_SELECT));
    assert!(module.bytes.contains(&opcodes::OP_I32_AND));
    assert!(module.bytes.contains(&opcodes::OP_I32_OR));
    assert!(module.bytes.contains(&opcodes::OP_I32_EQ));
    assert!(module.bytes.contains(&opcodes::OP_I32_NE));
    assert!(module.bytes.contains(&opcodes::OP_I64_XOR));
}

#[test]
fn cond_select_can_emit_always_condition() {
    let block = block(vec![instr_cond(Opcode::Csinc, 0b1110, 0, true)]);

    let module = Wasm64Compiler::compile(&block).expect("compile al csinc");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I32_CONST));
    assert!(module.bytes.contains(&opcodes::OP_SELECT));
}
