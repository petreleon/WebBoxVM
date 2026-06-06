use super::*;
use crate::arm64::decode;

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

#[test]
fn compiles_ands_register_and_bics_alias() {
    let block = block(vec![
        Instr {
            op: Opcode::AndsReg,
            rd: 4,
            rn: 5,
            rm: 6,
            cond: 0,
            sf: true,
            ..instr(Opcode::Nop, 0, 0, 0, 0, true)
        },
        Instr {
            op: Opcode::AndsReg,
            rd: ZERO_REGISTER_INDEX,
            rn: 1,
            rm: 2,
            cond: 4,
            sf: false,
            ..instr(Opcode::Nop, 0, 0, 0, 0, false)
        },
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile ands register");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.bytes.contains(&opcodes::OP_I64_AND));
    assert!(module.bytes.contains(&opcodes::OP_I64_XOR));
    assert!(module.bytes.contains(&opcodes::OP_I64_EQZ));
}

#[test]
fn compiles_observed_32_bit_eor_ror_form() {
    let instr = decode(0x4ac9_0949).expect("decode eor w9, w10, w9, ror #2");
    assert_eq!(instr.op, Opcode::EorReg);
    assert_eq!((instr.rd, instr.rn, instr.rm), (9, 10, 9));
    assert_eq!((instr.cond, instr.imm, instr.sf), (3, 2, false));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile eor ror");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHL));
    assert!(module.bytes.contains(&opcodes::OP_I64_XOR));
}
