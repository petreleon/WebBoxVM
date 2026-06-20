use super::*;

#[test]
fn decode_rejects_32_bit_bitfield_immediates_above_word_width() {
    let valid = decode(0x1300_3CE6).unwrap(); // sxth w6, w7
    assert_eq!(valid.op, Opcode::Sbfm);
    assert!(!valid.sf);
    assert_eq!(valid.rm, 0);
    assert_eq!(valid.imm, 15);

    assert!(decode(0x1320_3CE6).is_none()); // immr == 32
    assert!(decode(0x1300_BCE6).is_none()); // imms == 47
}

#[test]
fn decode_keeps_64_bit_sxtw_alias_valid() {
    assert_disarm64_mnemonic(0x9340_7C62, "sbfm");

    let instr = decode(0x9340_7C62).unwrap(); // sxtw x2, w3
    assert_eq!(instr.op, Opcode::Sxtw);
    assert!(instr.sf);
    assert_eq!(instr.rd, 2);
    assert_eq!(instr.rn, 3);
}
