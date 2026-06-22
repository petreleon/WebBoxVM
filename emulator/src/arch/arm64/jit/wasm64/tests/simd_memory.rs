use super::*;
use crate::arch::arm64::decode;

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
    assert!(
        module
            .bytes
            .windows(b"jitLoadGuest".len())
            .any(|w| w == b"jitLoadGuest")
    );
}

#[test]
fn compiles_observed_ld1_single_q_register() {
    let decoded = disarm64::decoder::decode(0x4c40_7041).expect("disarm64 decodes ld4");
    assert_eq!(format!("{:?}", decoded.mnemonic), "ld4");
    let instr = decode(0x4c40_7041).expect("decode observed ld1");
    assert_eq!(instr.op, Opcode::SimdLd1);
    assert_eq!((instr.rd, instr.rn, instr.rm), (1, 2, 0xff));
    assert_eq!((instr.imm, instr.cond, instr.size), (0, 1, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile ld1");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_CALL));
    assert!(
        module
            .bytes
            .windows(b"jitLoadGuest".len())
            .any(|w| w == b"jitLoadGuest")
    );
}

#[test]
fn compiles_observed_ld1_single_post_index_forms() {
    for (raw, regs, addr) in [
        (0x4cdf_7041, (1, 2, 0xfe), (16, 16)),
        (0x4cc8_7000, (0, 0, 8), (0, 16)),
        (0x0cdf_7004, (4, 0, 0xfe), (8, 8)),
    ] {
        let instr = decode(raw).expect("decode ld1 post-index");
        assert_eq!(instr.op, Opcode::SimdLd1);
        assert_eq!((instr.rd, instr.rn, instr.rm), regs);
        assert_eq!((instr.imm, instr.size), addr);

        let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile ld1 post");

        assert_eq!(module.guest_instr_count, 1);
        assert_eq!(module.exit_pc, 0x1004);
        assert!(module.bytes.contains(&opcodes::OP_CALL));
        assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
    }
}

#[test]
fn compiles_observed_simd_ldr_q_immediate() {
    let instr = decode(0x3dc0_0440).expect("decode ldr q0, [x2, #16]");
    assert_eq!(instr.op, Opcode::SimdLdr);
    assert_eq!((instr.rd, instr.rn, instr.rm), (0, 2, 0xff));
    assert_eq!((instr.imm, instr.size), (16, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile ldr q");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_CALL));
    assert!(
        module
            .bytes
            .windows(b"jitLoadGuest".len())
            .any(|w| w == b"jitLoadGuest")
    );
}

#[test]
fn compiles_observed_simd_ldr_q_preindex() {
    let instr = decode(0x3cc2_0c20).expect("decode ldr q0, [x1, #32]!");
    assert_eq!(instr.op, Opcode::SimdLdr);
    assert_eq!((instr.rd, instr.rn, instr.rm), (0, 1, 0xff));
    assert_eq!((instr.imm, instr.cond, instr.size), (32, 3, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile preindex ldr q");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(module.bytes.contains(&opcodes::OP_CALL));
    assert!(
        module
            .bytes
            .windows(b"jitLoadGuest".len())
            .any(|w| w == b"jitLoadGuest")
    );
}

#[test]
fn compiles_observed_simd_str_q_immediate() {
    let decoded = disarm64::decoder::decode(0x3d80_03fe).expect("disarm64 decodes str q");
    assert_eq!(format!("{:?}", decoded.mnemonic), "str");
    let instr = decode(0x3d80_03fe).expect("decode str q30, [sp]");

    assert_eq!(instr.op, Opcode::SimdStr);
    assert_eq!((instr.rd, instr.rn, instr.rm), (30, 31, 0xff));
    assert_eq!((instr.imm, instr.size), (0, 16));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile str q");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_CALL));
    assert!(
        module
            .bytes
            .windows(b"jitStoreGuest".len())
            .any(|w| w == b"jitStoreGuest")
    );
}

#[test]
fn compiles_observed_simd_ldp_q_pair_forms() {
    let cases = [
        (0xadc1_0821, (1, 1, 2), (32, 3)),
        (0xad40_0420, (0, 1, 1), (0, 2)),
    ];
    for (raw, regs, addr) in cases {
        let decoded = disarm64::decoder::decode(raw).expect("disarm64 decodes ldp");
        assert_eq!(format!("{:?}", decoded.mnemonic), "ldp");
        let instr = decode(raw).expect("decode observed ldp q pair");

        assert_eq!(instr.op, Opcode::SimdLdp);
        assert_eq!((instr.rd, instr.rn, instr.rm), regs);
        assert_eq!((instr.imm, instr.cond, instr.size), (addr.0, addr.1, 16));

        let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile simd ldp");

        assert_eq!(module.guest_instr_count, 1);
        assert!(module.bytes.contains(&opcodes::OP_CALL));
        assert!(
            module
                .bytes
                .windows(b"jitLoadGuest".len())
                .any(|w| w == b"jitLoadGuest")
        );
    }
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
    assert!(
        module
            .bytes
            .windows(b"jitStoreGuest".len())
            .any(|w| w == b"jitStoreGuest")
    );
}
