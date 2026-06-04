use super::*;

#[test]
fn decode_mte_gpr_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x9AC1_1000, Opcode::MteIrg, 0, 0, 1, "irg"),
        (0x9ADF_13FF, Opcode::MteIrg, 31, 31, 31, "irg"),
        (0x9ADF_1401, Opcode::MteGmi, 1, 0, 31, "gmi"),
        (0x9AC3_17E2, Opcode::MteGmi, 2, 31, 3, "gmi"),
    ];

    for (raw, op, rd, rn, rm, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, op);
        assert_eq!((instr.rd, instr.rn, instr.rm), (rd, rn, rm));
    }
}

#[test]
fn decode_mte_tag_memory_forms_cross_checked_with_disarm64() {
    let cases = [
        (0xD960_0000, Opcode::MteLdg, 0, 0, 0, 0, "ldg"),
        (0xD96F_F3E3, Opcode::MteLdg, 3, 31, 4080, 0, "ldg"),
        (0xD970_00A4, Opcode::MteLdg, 4, 5, -4096, 0, "ldg"),
        (0xD920_0800, Opcode::MteStg, 0, 0, 0, 0, "stg"),
        (0xD920_4C40, Opcode::MteStg, 0, 2, 64, 3, "stg"),
        (0xD920_1400, Opcode::MteStg, 0, 0, 16, 1, "stg"),
        (0xD960_0800, Opcode::MteStzg, 0, 0, 0, 0, "stzg"),
        (0xD9A0_0800, Opcode::MteSt2g, 0, 0, 0, 0, "st2g"),
        (0xD9E0_0800, Opcode::MteStz2g, 0, 0, 0, 0, "stz2g"),
    ];

    for (raw, op, rd, rn, imm, cond, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, op);
        assert_eq!((instr.rd, instr.rn, instr.cond), (rd, rn, cond));
        assert_eq!(instr.imm as i64, imm);
    }
}
