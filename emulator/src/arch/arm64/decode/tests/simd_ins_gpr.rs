use super::*;

#[test]
fn decode_simd_ins_general_preserves_element_size_and_lane() {
    let halfword = decode(0x4E0E_1D3F).unwrap();
    assert_disarm64_mnemonic(0x4E0E_1D3F, "ins");
    assert_eq!(halfword.op, Opcode::SimdInsGprLane);
    assert_eq!(halfword.rd, 31);
    assert_eq!(halfword.rn, 9);
    assert_eq!(halfword.imm, 3);
    assert_eq!(halfword.cond, 2);
    assert_eq!(halfword.size, 16);

    let doubleword = decode(0x4E18_1C3E).unwrap();
    assert_disarm64_mnemonic(0x4E18_1C3E, "ins");
    assert_eq!(doubleword.op, Opcode::SimdInsGprLane);
    assert_eq!(doubleword.rd, 30);
    assert_eq!(doubleword.rn, 1);
    assert_eq!(doubleword.imm, 1);
    assert_eq!(doubleword.cond, 8);

    let fmov_lane_alias = decode(0x9EAF_0060).unwrap();
    assert_disarm64_mnemonic(0x9EAF_0060, "fmov");
    assert_eq!(fmov_lane_alias.op, Opcode::SimdInsGprLane);
    assert_eq!(fmov_lane_alias.imm, 1);
    assert_eq!(fmov_lane_alias.cond, 8);
}
