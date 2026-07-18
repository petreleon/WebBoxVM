use super::*;

#[test]
fn decodes_hvc_imm16_in_legacy_and_primary_paths() {
    for imm16 in [0, 0x1234, 0xFFFF] {
        let raw = 0xD400_0002 | (imm16 << 5);
        assert_disarm64_mnemonic(raw, "hvc");

        let legacy = decode_legacy(raw).expect("legacy decoder should recognize HVC");
        assert_eq!(legacy.op, Opcode::Hvc);
        assert_eq!(legacy.imm, imm16 as u64);

        let primary = decode(raw).expect("primary decoder should recognize HVC");
        assert_eq!(primary.op, Opcode::Hvc);
        assert_eq!(primary.imm, imm16 as u64);
    }
}
