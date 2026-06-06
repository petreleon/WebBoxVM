use super::*;

#[test]
fn compiles_clz_64_and_32_bit_forms() {
    let block = block(vec![
        instr(Opcode::Clz, 1, 2, 0, 0, true),
        instr(Opcode::Clz, 3, 4, 0, 0, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile clz");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_CLZ));
    assert!(module.bytes.contains(&opcodes::OP_I32_CLZ));
    assert!(module.bytes.contains(&opcodes::OP_I64_EXTEND_I32_U));
}

#[test]
fn compiles_observed_rbit_64_form() {
    let decoded = disarm64::decoder::decode(0xdac0_0063).expect("disarm64 decodes rbit");
    assert_eq!(format!("{:?}", decoded.mnemonic), "rbit");
    let instr = crate::arm64::decode(0xdac0_0063).expect("decode rbit x3, x3");
    assert_eq!(instr.op, Opcode::Rbit);
    assert_eq!((instr.rd, instr.rn, instr.sf), (3, 3, true));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile rbit");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHL));
    assert!(module.bytes.contains(&opcodes::OP_I64_OR));
}
