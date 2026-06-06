use super::*;
use crate::arm64::decode;

#[test]
fn compiles_observed_addhn_8b_form() {
    let instr = decode(0x0e21_4022).expect("decode addhn v2.8b, v1.8h, v1.8h");
    assert_eq!(instr.op, Opcode::SimdAddhn);
    assert_eq!((instr.rd, instr.rn, instr.rm), (2, 1, 1));
    assert_eq!((instr.imm, instr.size), (1, 8));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile addhn 8b");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_ADD));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
}

#[test]
fn compiles_cross_checked_raddhn2_16b_form() {
    let instr = decode(0x6e2b_4149).expect("decode raddhn2 v9.16b, v10.8h, v11.8h");
    assert_eq!(instr.op, Opcode::SimdRaddhn2);
    assert_eq!((instr.imm, instr.size), (1, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile raddhn2 16b");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_ADD));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
}

#[test]
fn compiles_observed_shrn_8b_form() {
    let instr = decode(0x0f0c_8422).expect("decode shrn v2.8b, v1.8h, #4");
    assert_eq!(instr.op, Opcode::SimdShrn);
    assert_eq!((instr.rd, instr.rn, instr.imm, instr.cond), (2, 1, 4, 1));
    assert_eq!(instr.size, 8);

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile shrn 8b");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_AND));
}

#[test]
fn compiles_cross_checked_shrn2_16b_form() {
    let instr = decode(0x4f0a_87e2).expect("decode shrn2 v2.16b, v31.8h, #6");
    assert_eq!(instr.op, Opcode::SimdShrn2);
    assert_eq!((instr.rd, instr.rn, instr.imm, instr.cond), (2, 31, 6, 1));
    assert_eq!(instr.size, 16);

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile shrn2 16b");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
}
