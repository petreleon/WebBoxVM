use super::*;
use crate::arch::arm64::decode;

#[test]
fn compiles_observed_cmeq_zero_16b_form() {
    let instr = decode(0x4e20_9801).expect("decode cmeq v1.16b, v0.16b, #0");
    let block = block(vec![instr]);

    let module = Wasm64Compiler::compile(&block).expect("compile cmeq zero");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_MUL));
}

#[test]
fn compiles_observed_cmeq_zero_8b_form() {
    let instr = decode(0x0e20_9800).expect("decode cmeq v0.8b, v0.8b, #0");
    assert_eq!(instr.op, Opcode::SimdCmeqZero);
    assert_eq!((instr.imm, instr.size), (1, 8));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile cmeq zero 8b");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_MUL));
}

#[test]
fn compiles_observed_cmeq_register_16b_form() {
    let instr = decode(0x6e20_8c23).expect("decode cmeq v3.16b, v1.16b, v0.16b");
    assert_eq!(instr.op, Opcode::SimdCmeqReg);
    assert_eq!((instr.rd, instr.rn, instr.rm), (3, 1, 0));
    assert_eq!((instr.imm, instr.size), (1, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile cmeq reg 16b");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_XOR));
    assert!(module.bytes.contains(&opcodes::OP_I64_MUL));
}

#[test]
fn compiles_observed_cmhs_register_16b_form() {
    let instr = decode(0x6e21_3c62).expect("decode cmhs v2.16b, v3.16b, v1.16b");
    assert_eq!(instr.op, Opcode::SimdCmhsReg);
    assert_eq!((instr.rd, instr.rn, instr.rm), (2, 3, 1));
    assert_eq!((instr.imm, instr.size), (1, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile cmhs reg 16b");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_GE_U));
    assert!(module.bytes.contains(&opcodes::OP_SELECT));
}

#[test]
fn compiles_cross_checked_cmhi_register_16b_form() {
    let instr = decode(0x6e22_3462).expect("decode cmhi v2.16b, v3.16b, v2.16b");
    assert_eq!(instr.op, Opcode::SimdCmhiReg);
    assert_eq!((instr.imm, instr.size), (1, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile cmhi reg 16b");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_LT_U));
    assert!(module.bytes.contains(&opcodes::OP_SELECT));
}

#[test]
fn cmeq_zero_rejects_unimplemented_lane_widths() {
    let block = block(vec![Instr {
        op: Opcode::SimdCmeqZero,
        rd: 1,
        rn: 0,
        rm: 0,
        imm: 2,
        sf: true,
        cond: 0,
        size: 16,
    }]);

    let err = Wasm64Compiler::compile(&block).expect_err("reject non-byte cmeq zero");
    assert!(matches!(err, WasmJitError::UnsupportedFirstOpcode { .. }));
}
