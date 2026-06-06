use super::*;
use crate::arm64::decode;

#[test]
fn compiles_observed_ld1_multi_two_q_registers() {
    let instr = decode(0x4c40_a021).expect("decode observed ld1 multi");
    assert_eq!(instr.op, Opcode::SimdLd1Multi);
    assert_eq!((instr.rd, instr.rn, instr.rm), (1, 1, 0xff));
    assert_eq!((instr.cond, instr.size), (2, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile ld1 multi");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_CALL));
    assert!(module
        .bytes
        .windows(b"jitLoadGuest".len())
        .any(|w| w == b"jitLoadGuest"));
}

#[test]
fn compiles_observed_simd_stp_q_registers_as_boundary() {
    let stp = decode(0xad03_07e0).expect("decode observed stp q0, q1");
    assert_eq!(stp.op, Opcode::SimdStp);
    assert_eq!((stp.rd, stp.rn, stp.rm), (0, 31, 1));
    assert_eq!((stp.imm, stp.cond, stp.size), (96, 2, 16));

    let block = block(vec![stp, instr(Opcode::Nop, 0, 0, 0, 0, true)]);
    let module = Wasm64Compiler::compile(&block).expect("compile simd stp");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_CALL));
    assert!(module
        .bytes
        .windows(b"jitStoreGuest".len())
        .any(|w| w == b"jitStoreGuest"));
}
