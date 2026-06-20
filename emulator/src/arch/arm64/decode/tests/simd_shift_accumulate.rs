use super::*;

#[test]
fn decode_simd_unsigned_shift_right_accumulate_cross_checked_with_disarm64() {
    let cases = [
        (0x6F3F_14BC, 28, 5, 4, 1, 16, "usra"),
        (0x6F7F_14BA, 26, 5, 8, 1, 16, "usra"),
        (0x6F09_17A0, 0, 29, 1, 7, 16, "usra"),
        (0x2F3F_14BC, 28, 5, 4, 1, 8, "usra"),
    ];

    for (raw, rd, rn, element_size, shift, vector_size, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::SimdUsra);
        assert_eq!((instr.rd, instr.rn), (rd, rn));
        assert_eq!(
            (instr.cond, instr.imm, instr.size),
            (element_size, shift, vector_size)
        );
    }
}
