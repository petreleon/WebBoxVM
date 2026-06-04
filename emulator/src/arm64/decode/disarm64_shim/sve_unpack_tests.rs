use super::*;

#[test]
fn maps_sve_unpack_mnemonics_by_encoding() {
    let cases = [
        (0x05F0_3BD9, Opcode::SveSunpklo),
        (0x05F1_3BDE, Opcode::SveSunpkhi),
        (0x05F2_3B78, Opcode::SveUunpklo),
        (0x05F3_3B7B, Opcode::SveUunpkhi),
        (0x0530_40E1, Opcode::SvePunpklo),
        (0x0531_40E2, Opcode::SvePunpkhi),
    ];

    for (raw, op) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode SVE unpack word");
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(op));
    }
}
