use super::*;
use crate::arm64::decode;

#[test]
fn compiles_observed_umaxp_16b_form() {
    let instr = decode(0x6e21_a422).expect("decode umaxp v2.16b, v1.16b, v1.16b");
    assert_eq!(instr.op, Opcode::SimdUmaxp);
    assert_eq!((instr.rd, instr.rn, instr.rm), (2, 1, 1));
    assert_eq!((instr.imm, instr.size), (1, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile umaxp bytes");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_SELECT));
    assert!(module.bytes.contains(&opcodes::OP_I64_GE_U));
}

#[test]
fn compiles_observed_uminp_16b_form() {
    let instr = decode(0x6e20_ac00).expect("decode uminp v0.16b, v0.16b, v0.16b");
    assert_eq!(instr.op, Opcode::SimdUminp);
    assert_eq!((instr.rd, instr.rn, instr.rm), (0, 0, 0));
    assert_eq!((instr.imm, instr.size), (1, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile uminp bytes");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_SELECT));
    assert!(module.bytes.contains(&opcodes::OP_I64_LT_U));
}

#[test]
fn unsigned_pairwise_rejects_unimplemented_lane_widths() {
    let block = block(vec![Instr {
        op: Opcode::SimdUminp,
        rd: 2,
        rn: 1,
        rm: 1,
        imm: 2,
        sf: true,
        cond: 0,
        size: 16,
    }]);

    let err = Wasm64Compiler::compile(&block).expect_err("reject non-byte pairwise");
    assert!(matches!(err, WasmJitError::UnsupportedFirstOpcode { .. }));
}
