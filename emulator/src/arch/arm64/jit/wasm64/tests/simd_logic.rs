use super::*;
use crate::arch::arm64::decode;

#[test]
fn compiles_observed_and_16b_form() {
    let instr = decode(0x4e20_1c21).expect("decode and v1.16b, v1.16b, v0.16b");
    assert_eq!(instr.op, Opcode::SimdAnd);
    assert_eq!((instr.rd, instr.rn, instr.rm, instr.size), (1, 1, 0, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile simd and");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_AND));
}

#[test]
fn compiles_simd_logical_register_family() {
    for raw in [
        0x4eb9_1f18,
        0x6e3e_1ffe,
        0x0e64_1fde,
        0x4ee1_1c00,
        0x2e7f_1de1,
        0x2ebf_1de0,
        0x2efe_1fe0,
    ] {
        let instr = decode(raw).expect("decode simd logical register");
        let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile simd logical");
        assert_eq!(module.guest_instr_count, 1);
    }
}

#[test]
fn simd_logical_rejects_unknown_vector_size() {
    let block = block(vec![Instr {
        op: Opcode::SimdAnd,
        rd: 1,
        rn: 1,
        rm: 0,
        imm: 0,
        sf: true,
        cond: 0,
        size: 4,
    }]);

    let err = Wasm64Compiler::compile(&block).expect_err("reject unsupported vector size");
    assert!(matches!(err, WasmJitError::UnsupportedFirstOpcode { .. }));
}
