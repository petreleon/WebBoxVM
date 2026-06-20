use super::*;

#[test]
fn decode_mops_copy_and_set_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x1901_0443, Opcode::MopsCpyFp, "cpyfp"),
        (0x1941_0443, Opcode::MopsCpyFm, "cpyfm"),
        (0x1981_0443, Opcode::MopsCpyFe, "cpyfe"),
        (0x1D01_0443, Opcode::MopsCpyP, "cpyp"),
        (0x1D41_0443, Opcode::MopsCpyM, "cpym"),
        (0x1D81_0443, Opcode::MopsCpyE, "cpye"),
        (0x19C1_0443, Opcode::MopsSetP, "setp"),
        (0x19C1_4443, Opcode::MopsSetM, "setm"),
        (0x19C1_8443, Opcode::MopsSetE, "sete"),
    ];

    for (raw, op, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, op);
        assert_eq!((instr.rd, instr.rm, instr.rn), (3, 1, 2));
    }
}
