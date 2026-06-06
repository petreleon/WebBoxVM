use super::*;

#[test]
fn compiles_cmp_immediate_and_register_flags() {
    let block = block(vec![
        instr(Opcode::CmpImm, 0, 1, 0, 7, true),
        instr(Opcode::Cmp, 31, 2, 3, 0, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile cmp flags");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_LOCAL_SET));
    assert!(module.bytes.contains(&opcodes::OP_I64_GE_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_EXTEND_I32_U));
}

#[test]
fn compiles_subs_immediate_result_and_flags() {
    let block = block(vec![
        instr(Opcode::SubsImm, 2, 1, 0, 7, true),
        instr(Opcode::SubsImm, ZERO_REGISTER_INDEX, 3, 0, 1, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile subs immediate flags");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_SUB));
    assert!(module.bytes.contains(&opcodes::OP_I64_GE_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
}

#[test]
fn unsupported_extended_cmp_ends_prefix() {
    let block = block(vec![
        instr(Opcode::CmpImm, 0, 1, 0, 7, true),
        Instr {
            cond: 0x8,
            ..instr(Opcode::Cmp, 31, 2, 3, 0, true)
        },
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile cmp prefix");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
}
