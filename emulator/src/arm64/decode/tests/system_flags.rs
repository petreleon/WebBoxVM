use super::*;

#[test]
fn decode_flag_system_instructions() {
    let cases = [
        (0xD500_401F, Opcode::Cfinv, "cfinv"),
        (0xBA01_0423, Opcode::Rmif, "rmif"),
        (0x3A00_082D, Opcode::Setf8, "setf8"),
        (0x3A00_482D, Opcode::Setf16, "setf16"),
    ];
    assert_decode_cases(&cases);

    let rmif = decode(0xBA01_0423).unwrap(); // rmif x1, #2, #3
    assert_eq!((rmif.rn, rmif.imm, rmif.cond), (1, 2, 3));

    assert_eq!(decode(0x3A00_082D).unwrap().rn, 1);
    assert_eq!(decode(0x3A00_482D).unwrap().rn, 1);
}
