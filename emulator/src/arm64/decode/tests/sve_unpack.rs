use super::*;

#[test]
fn decode_sve_vector_unpack_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x05F0_3BD9, Opcode::SveSunpklo, "sunpklo"),
        (0x05F1_3BDE, Opcode::SveSunpkhi, "sunpkhi"),
        (0x05F2_3B78, Opcode::SveUunpklo, "uunpklo"),
        (0x05F3_3B7B, Opcode::SveUunpkhi, "uunpkhi"),
    ]);

    let instr = decode(0x05F2_3B78).unwrap(); // uunpklo z24.d, z27.s
    assert_eq!(
        (instr.rd, instr.rn, instr.cond, instr.size),
        (24, 27, 0xFF, 8)
    );
    assert!(decode(0x0532_3B78).is_none());
}

#[test]
fn decode_sve_predicate_unpack_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x0530_40E1, Opcode::SvePunpklo, "punpklo"),
        (0x0531_40E2, Opcode::SvePunpkhi, "punpkhi"),
    ]);

    let instr = decode(0x0530_40E1).unwrap(); // punpklo p1.h, p7.b
    assert_eq!(
        (instr.rd, instr.rn, instr.cond, instr.size),
        (1, 7, 0xFF, 2)
    );
}
