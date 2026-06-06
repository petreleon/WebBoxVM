use super::*;

fn stp_instr(cond: u8, imm: u64) -> Instr {
    Instr {
        cond,
        size: 0,
        ..instr(Opcode::Stp, 6, 3, 7, imm, true)
    }
}

fn ldp_instr(op: Opcode, cond: u8, imm: u64) -> Instr {
    Instr {
        cond,
        size: 0,
        ..instr(op, 10, 1, 11, imm, true)
    }
}

#[test]
fn compiles_pair_load_as_two_helper_calls() {
    let block = block(vec![ldp_instr(Opcode::Ldp, 2, 48)]);

    let module = Wasm64Compiler::compile(&block).expect("compile ldp");
    let calls = module
        .bytes
        .iter()
        .filter(|&&byte| byte == opcodes::OP_CALL)
        .count();

    assert_eq!(module.guest_instr_count, 1);
    assert!(calls >= 2);
}

#[test]
fn compiles_ldpsw_with_sign_extension() {
    let block = block(vec![ldp_instr(Opcode::Ldpsw, 3, 64)]);

    let module = Wasm64Compiler::compile(&block).expect("compile ldpsw");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_S));
}

#[test]
fn compiles_pair_store_as_two_staged_store_calls() {
    let block = block(vec![stp_instr(2, 16)]);

    let module = Wasm64Compiler::compile(&block).expect("compile stp");
    let calls = module
        .bytes
        .iter()
        .filter(|&&byte| byte == opcodes::OP_CALL)
        .count();

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.windows(b"jitStoreGuest".len()).any(|w| w == b"jitStoreGuest"));
    assert!(calls >= 2);
}

#[test]
fn pair_store_stops_block_after_boundary() {
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        stp_instr(3, 64),
        instr(Opcode::AddImm, 1, 0, 0, 7, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile stp prefix");

    assert_eq!(module.guest_instr_count, 2);
    assert_eq!(module.exit_pc, 0x1008);
}
