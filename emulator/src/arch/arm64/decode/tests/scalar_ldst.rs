use super::*;

#[test]
fn decode_scalar_byte_halfword_load_store_cross_checked_with_disarm64() {
    let cases = [
        (0x3940_0C41, Opcode::Ldr, "ldrb", 1, false),
        (0x7940_0C83, Opcode::Ldr, "ldrh", 2, false),
        (0x785F_E18B, Opcode::Ldr, "ldurh", 2, false),
        (0x3980_1CC5, Opcode::LdrSign, "ldrsb", 1, false),
        (0x79C0_1107, Opcode::LdrSign, "ldrsh", 2, true),
        (0x38DF_F149, Opcode::LdrSign, "ldursb", 1, true),
        (0x789F_A041, Opcode::LdrSign, "ldursh", 2, false),
        (0xB89F_8083, Opcode::LdrSign, "ldursw", 4, true),
        (0x3900_25CD, Opcode::Str, "strb", 1, false),
        (0x7900_160F, Opcode::Str, "strh", 2, false),
        (0x381F_D251, Opcode::Str, "sturb", 1, false),
        (0x781F_C293, Opcode::Str, "sturh", 2, false),
    ];

    for (raw, expected, mnemonic, size, sf) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.size, size, "raw=0x{raw:08x}");
        assert_eq!(instr.sf, sf, "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_rcpc_unscaled_loads_cross_checked_with_disarm64() {
    let cases = [
        (0x1940_1022, Opcode::Ldapur, "ldapurb", 1, false, 2, 1, 1),
        (0x5940_2023, Opcode::Ldapur, "ldapurh", 2, false, 3, 1, 2),
        (0x9940_3024, Opcode::Ldapur, "ldapur", 4, false, 4, 1, 3),
        (0xD940_4025, Opcode::Ldapur, "ldapur", 8, true, 5, 1, 4),
        (0x19C0_5026, Opcode::Ldapurs, "ldapursb", 1, false, 6, 1, 5),
        (0x1980_5027, Opcode::Ldapurs, "ldapursb", 1, true, 7, 1, 5),
        (0x5980_6028, Opcode::Ldapurs, "ldapursh", 2, true, 8, 1, 6),
        (0x59C0_6029, Opcode::Ldapurs, "ldapursh", 2, false, 9, 1, 6),
        (0x9980_702A, Opcode::Ldapurs, "ldapursw", 4, true, 10, 1, 7),
        (
            0x595F_F020,
            Opcode::Ldapur,
            "ldapurh",
            2,
            false,
            0,
            1,
            (-1i64) as u64,
        ),
    ];

    for (raw, expected, mnemonic, size, sf, rd, rn, imm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(
            (instr.size, instr.sf, instr.rd, instr.rn, instr.imm),
            (size, sf, rd, rn, imm)
        );
    }
}

#[test]
fn decode_rcpc_unscaled_stores_cross_checked_with_disarm64() {
    let cases = [
        (0x1900_1022, "stlurb", 1, false, 2, 1, 1),
        (0x5900_2023, "stlurh", 2, false, 3, 1, 2),
        (0x9900_3024, "stlur", 4, false, 4, 1, 3),
        (0xD900_4025, "stlur", 8, true, 5, 1, 4),
        (0x591F_F020, "stlurh", 2, false, 0, 1, (-1i64) as u64),
    ];

    for (raw, mnemonic, size, sf, rd, rn, imm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::Stlur, "raw=0x{raw:08x}");
        assert_eq!(
            (instr.size, instr.sf, instr.rd, instr.rn, instr.imm),
            (size, sf, rd, rn, imm)
        );
    }
}

#[test]
fn decode_authenticated_loads_cross_checked_with_disarm64() {
    let cases = [
        (
            0xF87F_060D,
            Opcode::Ldraa,
            "ldraa",
            13,
            16,
            (-128i64) as u64,
            0,
        ),
        (
            0xF8FF_060D,
            Opcode::Ldrab,
            "ldrab",
            13,
            16,
            (-128i64) as u64,
            0,
        ),
        (0xF820_2C20, Opcode::Ldraa, "ldraa", 0, 1, 16, 3),
        (0xF8A0_2C20, Opcode::Ldrab, "ldrab", 0, 1, 16, 3),
    ];

    for (raw, expected, mnemonic, rd, rn, imm, cond) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(
            (instr.rd, instr.rn, instr.imm, instr.cond, instr.size),
            (rd, rn, imm, cond, 8)
        );
    }
}
