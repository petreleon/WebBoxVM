use super::*;
use crate::arm64::decode;

#[test]
fn compiles_observed_movi_16b_immediate() {
    let decoded = disarm64::decoder::decode(0x4f01_e664).expect("disarm64 decodes movi");
    assert_eq!(format!("{:?}", decoded.mnemonic), "movi");
    let instr = decode(0x4f01_e664).expect("decode observed movi");
    assert_eq!(instr.op, Opcode::SimdMovi);
    assert_eq!((instr.rd, instr.imm, instr.cond, instr.size), (4, 0x33, 0, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile movi");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
}
