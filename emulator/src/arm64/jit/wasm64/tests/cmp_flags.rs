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
fn compiles_observed_cmp_extended_register() {
    let instr = crate::arm64::decode(0x6b21_02df).expect("decode observed cmp");
    assert_eq!(instr.op, Opcode::Cmp);
    assert_eq!((instr.rn, instr.rm, instr.cond, instr.imm, instr.sf), (22, 1, 8, 0, false));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile cmp extended");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_AND));
    assert!(module.bytes.contains(&opcodes::OP_I64_SUB));
    assert!(module.bytes.contains(&opcodes::OP_I64_GE_U));
}

#[test]
fn compiles_observed_subs_register_result_and_flags() {
    let instr = crate::arm64::decode(0x6b04_0040).expect("decode subs w0, w2, w4");
    assert_eq!(instr.op, Opcode::Subs);
    assert_eq!((instr.rd, instr.rn, instr.rm, instr.sf), (0, 2, 4, false));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile subs register");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_SUB));
    assert!(module.bytes.contains(&opcodes::OP_I64_GE_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
}
