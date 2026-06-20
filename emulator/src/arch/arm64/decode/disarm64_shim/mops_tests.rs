use super::*;

#[test]
fn maps_mops_mnemonics_by_encoding() {
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

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode MOPS word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
