use super::*;

#[test]
fn decode_sve_addsub_immediate_and_predicated_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x2520_C020, Opcode::SveAddImm, "add"),
        (0x25A0_FFE3, Opcode::SveAddImm, "add"),
        (0x2521_C024, Opcode::SveSubImm, "sub"),
        (0x25E1_FFE7, Opcode::SveSubImm, "sub"),
        (0x04C0_0FFE, Opcode::SveAddPred, "add"),
        (0x04C1_0FBF, Opcode::SveSubPred, "sub"),
    ];
    assert_decode_cases(&cases);

    let add_imm = decode(0x25A0_FFE3).unwrap(); // add z3.s, z3.s, #0xff00
    assert_eq!(
        (add_imm.rd, add_imm.rn, add_imm.size, add_imm.imm),
        (3, 3, 4, 0xFF00)
    );

    let sub_pred = decode(0x04C1_0FBF).unwrap(); // sub z31.d, p3/m, z31.d, z29.d
    assert_eq!(
        (
            sub_pred.rd,
            sub_pred.rn,
            sub_pred.rm,
            sub_pred.cond,
            sub_pred.size
        ),
        (31, 31, 29, 3, 8)
    );

    assert!(decode(0x2520_E000).is_none());
}
