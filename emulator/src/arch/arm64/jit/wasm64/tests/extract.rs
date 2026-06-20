use super::*;

#[test]
fn compiles_extract_and_rotate_alias_forms() {
    let block = block(vec![
        Instr {
            rm: 7,
            imm: 25,
            ..instr(Opcode::Extr, 7, 7, 0, 0, false)
        },
        Instr {
            rm: 4,
            imm: 7,
            ..instr(Opcode::Extr, 2, 3, 0, 0, false)
        },
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile extr");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHL));
    assert!(module.bytes.contains(&opcodes::OP_I64_OR));
}
