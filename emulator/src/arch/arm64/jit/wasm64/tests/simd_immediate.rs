use super::*;
use crate::arch::arm64::decode;

#[test]
fn compiles_observed_movi_16b_immediate() {
    let decoded = disarm64::decoder::decode(0x4f01_e664).expect("disarm64 decodes movi");
    assert_eq!(format!("{:?}", decoded.mnemonic), "movi");
    let instr = decode(0x4f01_e664).expect("decode observed movi");
    assert_eq!(instr.op, Opcode::SimdMovi);
    assert_eq!(
        (instr.rd, instr.imm, instr.cond, instr.size),
        (4, 0x33, 0, 16)
    );

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile movi");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
}

#[test]
fn compiles_observed_bic_16b_immediate() {
    let decoded = disarm64::decoder::decode(0x6f07_9604).expect("disarm64 decodes bic");
    assert_eq!(format!("{:?}", decoded.mnemonic), "bic");
    let instr = decode(0x6f07_9604).expect("decode observed bic");
    assert_eq!(instr.op, Opcode::SimdBicImm);
    assert_eq!(
        (instr.rd, instr.imm, instr.cond, instr.size),
        (4, 0xf0, 2, 16)
    );

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile bic");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_AND));
    assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
    assert!(!module.uses_guest_helpers);
}

#[test]
fn compiles_bic_8b_immediate_with_high_half_clear() {
    let instr = Instr {
        op: Opcode::SimdBicImm,
        rd: 3,
        rn: 0,
        rm: 0,
        imm: 0x3300,
        sf: true,
        cond: 2,
        size: 8,
    };

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile bic 8b");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_AND));
    assert!(
        module
            .bytes
            .windows(3)
            .any(|bytes| bytes == [opcodes::OP_I64_CONST, 0, opcodes::OP_I64_STORE])
    );
}
