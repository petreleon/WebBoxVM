use super::*;

#[test]
fn compiles_observed_adds_register_result_and_flags() {
    let instr = crate::arch::arm64::decode(0xab02_0022).expect("decode adds x2, x1, x2");
    assert_eq!(instr.op, Opcode::Adds);

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile adds register");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_ADD));
    assert!(module.bytes.contains(&opcodes::OP_I64_LT_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
}

#[test]
fn compiles_observed_adds_immediate_cmn_alias() {
    let instr = crate::arch::arm64::decode(0x3100_049f).expect("decode adds wzr, w4, #1");
    assert_eq!(instr.op, Opcode::AddsImm);
    assert_eq!(instr.rd, ZERO_REGISTER_INDEX);

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile adds immediate");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_ADD));
    assert!(module.bytes.contains(&opcodes::OP_I64_LT_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_EXTEND_I32_U));
}

#[test]
fn rejects_unallocated_adds_register_shift() {
    let err = Wasm64Compiler::compile(&block(vec![Instr {
        op: Opcode::Adds,
        cond: 3,
        imm: 0,
        ..instr(Opcode::Nop, 1, 2, 3, 0, true)
    }]))
    .expect_err("reject adds ror");

    assert!(matches!(err, WasmJitError::UnsupportedFirstOpcode { .. }));
}
