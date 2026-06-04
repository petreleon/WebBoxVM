use super::*;

#[test]
fn decode_udf_forms_cross_checked_with_disarm64() {
    let cases = [(0x0000_0000, 0), (0x0000_1234, 0x1234)];

    for (raw, imm) in cases {
        assert_disarm64_mnemonic(raw, "udf");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::Udf);
        assert_eq!(instr.imm, imm);
    }
}
