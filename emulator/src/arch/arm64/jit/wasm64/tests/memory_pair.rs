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
fn compiles_pair_store_with_pair_helper_call() {
    let block = block(vec![stp_instr(2, 16)]);

    let module = Wasm64Compiler::compile(&block).expect("compile stp");
    let pair_helper_calls = module
        .bytes
        .windows([opcodes::OP_CALL, 6].len())
        .filter(|&w| w == [opcodes::OP_CALL, 6])
        .count();

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(pair_helper_calls, 1);
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

#[test]
fn compiles_observed_stxp_with_exclusive_pair_helper() {
    let instr = crate::arch::arm64::decode(0xc827_0c82).expect("decode observed stxp");
    assert_eq!(instr.op, Opcode::Stxp);
    assert_eq!((instr.imm, instr.rd, instr.rm, instr.rn), (7, 2, 3, 4));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile stxp");

    assert_eq!(module.guest_instr_count, 1);
    assert!(
        module
            .bytes
            .windows(b"jitStoreExclusivePair".len())
            .any(|w| w == b"jitStoreExclusivePair")
    );
}

#[test]
fn compiles_observed_ldxr_with_exclusive_load_helper() {
    let instr = crate::arch::arm64::decode(0x885f_7c60).expect("decode observed ldxr");
    assert_eq!(instr.op, Opcode::Ldxr);
    assert_eq!((instr.rd, instr.rn, instr.size, instr.sf), (0, 3, 4, false));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile ldxr");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.uses_guest_helpers);
    assert!(
        module
            .bytes
            .windows(b"jitLoadExclusive".len())
            .any(|w| w == b"jitLoadExclusive")
    );
}

#[test]
fn compiles_observed_stxr_with_exclusive_store_helper() {
    let instr = crate::arch::arm64::decode(0x8801_7f40).expect("decode observed stxr");
    assert_eq!(instr.op, Opcode::Stxr);
    assert_eq!((instr.imm, instr.rd, instr.rn, instr.size), (1, 0, 26, 4));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile stxr");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.uses_guest_helpers);
    assert!(
        module
            .bytes
            .windows(b"jitStoreExclusive".len())
            .any(|w| w == b"jitStoreExclusive")
    );
}

#[test]
fn exclusive_store_stops_block_after_boundary() {
    let stxr = crate::arch::arm64::decode(0x8801_7f40).expect("decode observed stxr");
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        stxr,
        instr(Opcode::AddImm, 1, 0, 0, 7, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile stxr prefix");

    assert_eq!(module.guest_instr_count, 2);
    assert_eq!(module.exit_pc, 0x1008);
}

#[test]
fn exclusive_load_stops_block_after_boundary() {
    let ldxr = crate::arch::arm64::decode(0x885f_7c60).expect("decode observed ldxr");
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        ldxr,
        instr(Opcode::AddImm, 1, 0, 0, 7, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile ldxr prefix");

    assert_eq!(module.guest_instr_count, 2);
    assert_eq!(module.exit_pc, 0x1008);
}

#[test]
fn exclusive_pair_store_stops_block_after_boundary() {
    let stxp = crate::arch::arm64::decode(0xc827_0c82).expect("decode observed stxp");
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        stxp,
        instr(Opcode::AddImm, 1, 0, 0, 7, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile stxp prefix");

    assert_eq!(module.guest_instr_count, 2);
    assert_eq!(module.exit_pc, 0x1008);
}
