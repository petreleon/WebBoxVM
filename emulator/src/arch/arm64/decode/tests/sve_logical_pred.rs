use super::*;

#[test]
fn decode_sve_logical_vector_and_predicate_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x0423_33BD, Opcode::SveAndVec, "and"),
        (0x04D8_0FE1, Opcode::SveOrrPred, "orr"),
        (0x04D9_0FE1, Opcode::SveEorPred, "eor"),
        (0x04DA_0FDF, Opcode::SveAndPred, "and"),
        (0x250F_4413, Opcode::SvePredBic, "bic"),
        (0x2540_4010, Opcode::SvePredBic, "bics"),
        (0x2500_4242, Opcode::SvePredEor, "eor"),
        (0x2540_4242, Opcode::SvePredEor, "eors"),
        (0x25C2_4073, Opcode::SvePredOrn, "orns"),
        (0x2580_4200, Opcode::SvePredNor, "nor"),
        (0x2580_4210, Opcode::SvePredNand, "nand"),
    ];
    assert_decode_cases(&cases);

    let and_pred = decode(0x04DA_0FDF).unwrap(); // and z31.d, p3/m, z31.d, z30.d
    assert_eq!(
        (
            and_pred.rd,
            and_pred.rn,
            and_pred.rm,
            and_pred.cond,
            and_pred.size
        ),
        (31, 31, 30, 3, 8)
    );

    let eors = decode(0x2540_4242).unwrap(); // eors p2.b, p0/z, p2.b, p0.b
    assert_eq!(
        (eors.rd, eors.rn, eors.rm, eors.cond, eors.size, eors.sf),
        (2, 2, 0, 0, 1, true)
    );

    let bic = decode(0x250F_4413).unwrap(); // bic p3.b, p1/z, p0.b, p15.b
    assert_eq!(
        (bic.rd, bic.rn, bic.rm, bic.cond, bic.size, bic.sf),
        (3, 0, 15, 1, 1, false)
    );
}
