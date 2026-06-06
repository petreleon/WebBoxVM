use super::*;

#[test]
fn compiles_madd_and_msub_scalar_forms() {
    let block = block(vec![
        Instr {
            op: Opcode::Madd,
            cond: 2,
            ..instr(Opcode::Nop, 0, 0, 3, 0, false)
        },
        Instr {
            op: Opcode::Msub,
            cond: 23,
            ..instr(Opcode::Nop, 20, 22, 21, 0, true)
        },
        Instr {
            op: Opcode::Madd,
            cond: ZERO_REGISTER_INDEX,
            ..instr(Opcode::Nop, 3, 4, 5, 0, true)
        },
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile madd/msub");

    assert_eq!(module.guest_instr_count, 3);
    assert!(module.bytes.contains(&opcodes::OP_I64_MUL));
    assert!(module.bytes.contains(&opcodes::OP_I64_ADD));
    assert!(module.bytes.contains(&opcodes::OP_I64_SUB));
}

#[test]
fn compiles_signed_and_unsigned_long_madd_forms() {
    let block = block(vec![
        Instr {
            op: Opcode::Madd,
            cond: 4,
            size: 1,
            ..instr(Opcode::Nop, 0, 2, 3, 0, true)
        },
        Instr {
            op: Opcode::Madd,
            cond: 7,
            size: 2,
            ..instr(Opcode::Nop, 4, 5, 6, 0, true)
        },
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile long madd");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_EXTEND_I32_S));
}

#[test]
fn compiles_observed_udiv_with_zero_divisor_guard() {
    let instr = crate::arm64::decode(0x1ac1_0841).expect("decode udiv w1, w2, w1");
    assert_eq!(instr.op, Opcode::Udiv);
    assert!(!instr.sf);

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile udiv");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_DIV_U));
    assert!(module.bytes.contains(&opcodes::OP_SELECT));
    assert!(module.bytes.contains(&opcodes::OP_I64_NE));
}

#[test]
fn compiles_observed_umulh_high_half_product() {
    for raw in [0x9bc4_7cc6, 0x9bd8_7ef7, 0x9bc3_7c84] {
        let decoded = disarm64::decoder::decode(raw).expect("disarm64 decodes umulh");
        assert_eq!(format!("{:?}", decoded.mnemonic), "umulh");
        let instr = crate::arm64::decode(raw).expect("decode observed umulh");

        assert_eq!(instr.op, Opcode::Umulh);
        assert!(instr.sf);

        let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile umulh");

        assert_eq!(module.guest_instr_count, 1);
        assert!(module.bytes.contains(&opcodes::OP_I64_MUL));
        assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
        assert!(module.bytes.contains(&opcodes::OP_I64_AND));
    }
}
