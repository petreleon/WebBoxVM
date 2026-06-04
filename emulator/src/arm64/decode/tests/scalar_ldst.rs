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
fn decode_authenticated_loads_cross_checked_with_disarm64() {
    let cases = [
        (0xF87F_060D, Opcode::Ldraa, "ldraa", 13, 16, (-128i64) as u64, 0),
        (0xF8FF_060D, Opcode::Ldrab, "ldrab", 13, 16, (-128i64) as u64, 0),
        (0xF820_2C20, Opcode::Ldraa, "ldraa", 0, 1, 16, 3),
        (0xF8A0_2C20, Opcode::Ldrab, "ldrab", 0, 1, 16, 3),
    ];

    for (raw, expected, mnemonic, rd, rn, imm, cond) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!((instr.rd, instr.rn, instr.imm, instr.cond, instr.size), (rd, rn, imm, cond, 8));
    }
}
