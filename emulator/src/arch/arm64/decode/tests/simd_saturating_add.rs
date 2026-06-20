use super::*;

#[test]
fn decode_simd_uqadd_vector_cross_checked_with_disarm64() {
    let cases = [
        (0x6EBB_0FDE, 30, 30, 27, 4, 16, "uqadd"),
        (0x6EF9_0F9C, 28, 28, 25, 8, 16, "uqadd"),
        (0x2EBB_0FDE, 30, 30, 27, 4, 8, "uqadd"),
    ];

    for (raw, rd, rn, rm, element_size, vector_size, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::SimdUqadd);
        assert_eq!((instr.rd, instr.rn, instr.rm), (rd, rn, rm));
        assert_eq!((instr.imm, instr.size), (element_size, vector_size));
    }

    assert!(decode(0x2EF9_0F9C).is_none());
}
