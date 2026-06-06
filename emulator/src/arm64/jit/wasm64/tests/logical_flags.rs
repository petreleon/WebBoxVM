use super::*;

#[test]
fn compiles_ands_immediate_and_updates_pstate() {
    let block = block(vec![
        Instr {
            op: Opcode::AndsImm,
            rd: 2,
            rn: 1,
            imm: 0xff,
            sf: true,
            ..instr(Opcode::Nop, 0, 0, 0, 0, true)
        },
        instr(Opcode::AddImm, 3, 2, 0, 1, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile ands immediate");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_EQZ));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHL));
}

#[test]
fn compiles_tst_alias_ands_immediate() {
    let block = block(vec![Instr {
        op: Opcode::AndsImm,
        rd: ZERO_REGISTER_INDEX,
        rn: 1,
        imm: 0xffff,
        sf: false,
        ..instr(Opcode::Nop, 0, 0, 0, 0, false)
    }]);

    let module = Wasm64Compiler::compile(&block).expect("compile tst alias");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
}
