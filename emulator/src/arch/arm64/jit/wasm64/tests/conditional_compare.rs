use super::*;

#[test]
fn compiles_conditional_compare_family() {
    let block = block(vec![
        Instr {
            cond: 0,
            size: 1,
            ..instr(Opcode::Ccmp, 0, 14, 0, 0, true)
        },
        Instr {
            cond: 1,
            size: 0,
            ..instr(Opcode::Ccmn, 0, 2, 3, 0xf, false)
        },
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile ccmp/ccmn");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_SELECT));
    assert!(module.bytes.contains(&opcodes::OP_I64_ADD));
    assert!(module.bytes.contains(&opcodes::OP_I64_SUB));
    assert!(module.bytes.contains(&opcodes::OP_I64_LT_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_GE_U));
}
