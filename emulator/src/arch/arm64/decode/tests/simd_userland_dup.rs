use super::*;

#[test]
fn decode_simd_userland_dup_lane_forms() {
    let dup_scalar_s_lane1 = decode(0x5E0C_07DF).unwrap();
    assert_eq!(dup_scalar_s_lane1.op, Opcode::SimdDupElem);
    assert_eq!(dup_scalar_s_lane1.rd, 31);
    assert_eq!(dup_scalar_s_lane1.rn, 30);
    assert_eq!(dup_scalar_s_lane1.imm, 1);
    assert_eq!(dup_scalar_s_lane1.cond, 4);
    assert_eq!(dup_scalar_s_lane1.size, 4);
    let dup_scalar_s_lane2 = decode(0x5E14_07DF).unwrap();
    assert_eq!(dup_scalar_s_lane2.op, Opcode::SimdDupElem);
    assert_eq!(dup_scalar_s_lane2.rd, 31);
    assert_eq!(dup_scalar_s_lane2.rn, 30);
    assert_eq!(dup_scalar_s_lane2.imm, 2);
    assert_eq!(dup_scalar_s_lane2.cond, 4);
    assert_eq!(dup_scalar_s_lane2.size, 4);
    let dup_scalar_double = decode(0x5E18_0694).unwrap();
    assert_eq!(dup_scalar_double.op, Opcode::SimdDupElem);
    assert_eq!(dup_scalar_double.rd, 20);
    assert_eq!(dup_scalar_double.rn, 20);
    assert_eq!(dup_scalar_double.imm, 1);
    assert_eq!(dup_scalar_double.cond, 8);
    assert_eq!(dup_scalar_double.size, 8);
    let dup_scalar_s_lane3 = decode(0x5E1C_071E).unwrap();
    assert_eq!(dup_scalar_s_lane3.op, Opcode::SimdDupElem);
    assert_eq!(dup_scalar_s_lane3.rd, 30);
    assert_eq!(dup_scalar_s_lane3.rn, 24);
    assert_eq!(dup_scalar_s_lane3.imm, 3);
    assert_eq!(dup_scalar_s_lane3.cond, 4);
    assert_eq!(dup_scalar_s_lane3.size, 4);
    let dup_scalar_s_lane3_alt = decode(0x5E1C_073A).unwrap();
    assert_eq!(dup_scalar_s_lane3_alt.op, Opcode::SimdDupElem);
    assert_eq!(dup_scalar_s_lane3_alt.rd, 26);
    assert_eq!(dup_scalar_s_lane3_alt.rn, 25);
    assert_eq!(dup_scalar_s_lane3_alt.imm, 3);
    assert_eq!(dup_scalar_s_lane3_alt.cond, 4);
    assert_eq!(dup_scalar_s_lane3_alt.size, 4);
}
