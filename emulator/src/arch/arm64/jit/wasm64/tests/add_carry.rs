use super::*;

#[test]
fn compiles_adc_sbc_register_forms() {
    let block = block(vec![
        instr(Opcode::Adc, 8, 8, 4, 0, true),
        instr(Opcode::Sbc, 2, 3, 5, 0, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile add carry");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_ADD));
    assert!(module.bytes.contains(&opcodes::OP_I64_XOR));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
}

#[test]
fn compiles_adcs_sbcs_flag_forms() {
    let block = block(vec![
        instr(Opcode::Adcs, 9, 10, 11, 0, true),
        instr(Opcode::Sbcs, ZERO_REGISTER_INDEX, 12, 13, 0, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile add carry flags");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_LT_U));
    assert!(module.bytes.contains(&opcodes::OP_I32_OR));
    assert!(module.bytes.contains(&opcodes::OP_I64_EXTEND_I32_U));
}
