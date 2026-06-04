use super::*;

#[test]
fn decode_simd_unsigned_widen_multiply_vectors_cross_checked_with_disarm64() {
    let cases = [
        (0x2E23_C041, Opcode::SimdUmull, "umull", 1, false, 0),
        (0x6E63_C041, Opcode::SimdUmull, "umull2", 2, true, 0),
        (0x2F87_A9D7, Opcode::SimdUmullElem, "umull", 4, false, 2),
        (0x6F87_A9D7, Opcode::SimdUmullElem, "umull2", 4, true, 2),
        (0x2E65_80A4, Opcode::SimdUmlalVec, "umlal", 2, false, 0),
        (0x6EA7_80E6, Opcode::SimdUmlalVec, "umlal2", 4, true, 0),
    ];

    for (raw, op, mnemonic, element_size, high_half, imm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, op, "raw=0x{raw:08x}");
        assert_eq!(instr.cond, element_size);
        assert_eq!(instr.sf, high_half);
        assert_eq!(instr.imm, imm);
        assert_eq!(instr.size, 16);
    }

    assert!(decode(0x2EE0_C000).is_none());
}
