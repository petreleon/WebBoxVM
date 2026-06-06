use super::*;
use crate::arm64::decode;

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
