use super::*;

#[test]
fn decode_simd_signed_variable_shift_cross_checked_with_disarm64() {
    let cases = [
        (0x4EBD_46F7, 23, 23, 29, 4, 16),
        (0x0E3A_4442, 2, 2, 26, 1, 8),
        (0x5EFF_47DF, 31, 30, 31, 8, 8),
    ];
    for (raw, rd, rn, rm, element_size, vector_size) in cases {
        assert_disarm64_mnemonic(raw, "sshl");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::SimdSshl, "raw=0x{raw:08x}");
        assert_eq!(instr.rd, rd);
        assert_eq!(instr.rn, rn);
        assert_eq!(instr.rm, rm);
        assert_eq!(instr.imm, element_size);
        assert_eq!(instr.size, vector_size);
    }
}
