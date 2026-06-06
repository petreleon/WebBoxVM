use super::*;
use crate::arm64::decode;

#[test]
fn compiles_observed_dup_gpr_4s_form() {
    let instr = decode(0x4e04_0c40).expect("decode dup v0.4s, w2");
    assert_eq!(instr.op, Opcode::SimdDupByte);
    assert_eq!((instr.rd, instr.rn, instr.cond, instr.size), (0, 2, 4, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile dup gpr");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHL));
    assert!(module.bytes.contains(&opcodes::OP_I64_OR));
}

#[test]
fn dup_gpr_rejects_unimplemented_lane_widths() {
    let block = block(vec![Instr {
        op: Opcode::SimdDupByte,
        rd: 0,
        rn: 2,
        rm: 0,
        imm: 0,
        sf: true,
        cond: 2,
        size: 16,
    }]);

    let err = Wasm64Compiler::compile(&block).expect_err("reject non-s dup");
    assert!(matches!(err, WasmJitError::UnsupportedFirstOpcode { .. }));
}
