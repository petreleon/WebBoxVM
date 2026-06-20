use super::*;

#[test]
fn decode_simd_minmax_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x0E20_6C20, Opcode::SimdSminVec, "smin"),
        (0x4EA0_6C20, Opcode::SimdSminVec, "smin"),
        (0x0E20_A420, Opcode::SimdSmaxp, "smaxp"),
        (0x4E60_A420, Opcode::SimdSmaxp, "smaxp"),
        (0x0E20_AC20, Opcode::SimdSminp, "sminp"),
        (0x4EA0_AC20, Opcode::SimdSminp, "sminp"),
        (0x2E20_A420, Opcode::SimdUmaxp, "umaxp"),
    ];
    assert_decode_cases(&cases);

    let smin_words = decode(0x4EA0_6C20).unwrap();
    assert_eq!((smin_words.rd, smin_words.rn, smin_words.rm), (0, 1, 0));
    assert_eq!((smin_words.imm, smin_words.size), (4, 16));
    let umaxp_bytes = decode(0x2E20_A420).unwrap();
    assert_eq!((umaxp_bytes.imm, umaxp_bytes.size), (1, 8));
    for invalid in [0x0EE0_6C20, 0x0EE0_A420, 0x2EE0_A420, 0x2EE0_AC20] {
        assert!(decode(invalid).is_none(), "raw=0x{invalid:08x}");
    }
}
