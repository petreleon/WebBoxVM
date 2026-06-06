use super::*;

#[test]
fn compiles_add_sub_extended_register_forms() {
    let block = block(vec![
        Instr {
            op: Opcode::AddExt,
            cond: 2,
            imm: 4,
            ..instr(Opcode::Nop, 17, 0, 8, 0, true)
        },
        Instr {
            op: Opcode::SubExt,
            cond: 6,
            imm: 2,
            ..instr(Opcode::Nop, 4, 5, 6, 0, false)
        },
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile extended add/sub");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_ADD));
    assert!(module.bytes.contains(&opcodes::OP_I64_SUB));
    assert!(module.bytes.contains(&opcodes::OP_I64_EXTEND_I32_S));
}

#[test]
fn rejects_invalid_extended_shift_amount() {
    let block = block(vec![Instr {
        op: Opcode::AddExt,
        cond: 2,
        imm: 7,
        ..instr(Opcode::Nop, 1, 2, 3, 0, true)
    }]);

    let err = Wasm64Compiler::compile(&block).expect_err("reject invalid shift");

    assert!(matches!(err, WasmJitError::UnsupportedFirstOpcode { .. }));
}
