use super::*;

#[test]
fn decode_sve_word_load_store_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0xA540_AC00, Opcode::SveLd1w, "ld1w"),
        (0xA560_AC00, Opcode::SveLd1w, "ld1w"),
        (0xE540_EC00, Opcode::SveSt1w, "st1w"),
        (0xE560_EC00, Opcode::SveSt1w, "st1w"),
        (0x8526_407D, Opcode::SveLd1w, "ld1w"),
        (0x857C_4001, Opcode::SveLd1w, "ld1w"),
        (0xC57C_C001, Opcode::SveLd1w, "ld1w"),
    ]);

    let ld_imm = decode(0xA541_ACA7).unwrap(); // ld1w { z7.s }, p3/z, [x5, #1, mul vl]
    assert_eq!((ld_imm.rd, ld_imm.rn, ld_imm.rm), (7, 5, 0xFF));
    assert_eq!((ld_imm.cond, ld_imm.imm as i64, ld_imm.size), (3, 1, 4));

    let st_d = decode(0xE560_EC00).unwrap(); // st1w { z0.d }, p3, [x0]
    assert_eq!((st_d.op, st_d.cond, st_d.size), (Opcode::SveSt1w, 3, 8));

    let gather_uxtw = decode(0x8526_407D).unwrap();
    assert_eq!((gather_uxtw.rd, gather_uxtw.rn, gather_uxtw.rm), (29, 3, 6));
    assert_eq!(
        (gather_uxtw.cond, gather_uxtw.size, gather_uxtw.sf),
        (0, 4, false)
    );

    let gather_sxtw = decode(0x857C_4001).unwrap();
    assert_eq!(
        (gather_sxtw.rd, gather_sxtw.rm, gather_sxtw.sf),
        (1, 28, true)
    );

    let gather_d = decode(0xC57C_C001).unwrap();
    assert_eq!((gather_d.rd, gather_d.rm, gather_d.size), (1, 28, 8));
}
