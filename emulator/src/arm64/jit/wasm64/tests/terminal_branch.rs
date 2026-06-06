use super::*;

#[test]
fn compiles_cbz_as_dynamic_terminal() {
    let block = block(vec![Instr {
        imm: 8,
        ..instr(Opcode::Cbz, 0, 0, 0, 0, true)
    }]);

    let module = Wasm64Compiler::compile(&block).expect("compile cbz");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.dynamic_exit);
    assert_eq!(module.exit_pc, 0x1004);
    assert_eq!(module.alternate_exit_pc, 0x1008);
    assert!(module.bytes.contains(&opcodes::OP_SELECT));
}

#[test]
fn compiles_bcond_with_negative_target() {
    let block = block(vec![Instr {
        op: Opcode::BCond,
        cond: 0b1010,
        imm: (-4i64) as u64,
        ..instr(Opcode::BCond, 0, 0, 0, 0, true)
    }]);

    let module = Wasm64Compiler::compile(&block).expect("compile b.cond");

    assert!(module.dynamic_exit);
    assert_eq!(module.exit_pc, 0x1004);
    assert_eq!(module.alternate_exit_pc, 0x0ffc);
    assert!(module.bytes.contains(&opcodes::OP_I32_EQ));
}

#[test]
fn compiles_test_bit_branches_as_dynamic_terminals() {
    for op in [Opcode::Tbz, Opcode::Tbnz] {
        let block = block(vec![Instr {
            op,
            cond: 7,
            imm: 12,
            ..instr(op, 3, 0, 0, 0, false)
        }]);

        let module = Wasm64Compiler::compile(&block).expect("compile tbz/tbnz");

        assert_eq!(module.guest_instr_count, 1);
        assert!(module.dynamic_exit);
        assert_eq!(module.alternate_exit_pc, 0x100c);
        assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
    }
}

#[test]
fn compiles_unconditional_branch_as_static_terminal() {
    let block = block(vec![Instr {
        imm: (-8i64) as u64,
        ..instr(Opcode::B, 0, 0, 0, 0, true)
    }]);

    let module = Wasm64Compiler::compile(&block).expect("compile b");

    assert_eq!(module.guest_instr_count, 1);
    assert!(!module.dynamic_exit);
    assert_eq!(module.exit_pc, 0x0ff8);
    assert_eq!(module.alternate_exit_pc, 0x0ff8);
}

#[test]
fn compiles_branch_with_link_and_sets_lr() {
    let block = block(vec![Instr {
        imm: 0x20,
        ..instr(Opcode::Bl, 0, 0, 0, 0, true)
    }]);

    let module = Wasm64Compiler::compile(&block).expect("compile bl");

    assert_eq!(module.guest_instr_count, 1);
    assert!(!module.dynamic_exit);
    assert_eq!(module.exit_pc, 0x1020);
    assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
}
