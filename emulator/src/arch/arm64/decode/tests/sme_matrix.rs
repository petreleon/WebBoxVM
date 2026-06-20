use super::*;

#[test]
fn decode_sme_smlal_indexed_vectors_cross_checked_with_disarm64() {
    let cases = [
        (0xC1D2_1300, Opcode::SmeSmlal, "smlal"),
        (0xC1D2_1700, Opcode::SmeSmlal, "smlal"),
        (0xC1D2_9301, Opcode::SmeSmlal, "smlal"),
    ];
    assert_decode_cases(&cases);

    let z2 = decode(0xC1D2_1300).unwrap();
    assert_eq!(z2.op, Opcode::SmeSmlal);
    assert_eq!(z2.rd, 0);
    assert_eq!(z2.rn, 24);
    assert_eq!(z2.rm, 2);
    assert_eq!(z2.imm, 0);
    assert_eq!(z2.cond, 0);
    assert_eq!(z2.size, 2);

    let indexed = decode(0xC1D2_1700).unwrap();
    assert_eq!(indexed.imm, 2);

    let z4 = decode(0xC1D2_9301).unwrap();
    assert_eq!(z4.rd, 1);
    assert_eq!(z4.rn, 24);
    assert_eq!(z4.size, 4);
}
