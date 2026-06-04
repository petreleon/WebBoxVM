use super::*;

#[test]
fn decode_sha1_sha256_crypto_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x5E06_00A2, Opcode::SimdSha1C, "sha1c", 6),
        (0x5E06_10A2, Opcode::SimdSha1P, "sha1p", 6),
        (0x5E06_20A2, Opcode::SimdSha1M, "sha1m", 6),
        (0x5E06_30A2, Opcode::SimdSha1Su0, "sha1su0", 6),
        (0x5E06_40A2, Opcode::SimdSha256H, "sha256h", 6),
        (0x5E06_50A2, Opcode::SimdSha256H2, "sha256h2", 6),
        (0x5E06_60A2, Opcode::SimdSha256Su1, "sha256su1", 6),
        (0x5E28_18A2, Opcode::SimdSha1Su1, "sha1su1", 0),
    ];

    for (raw, op, mnemonic, rm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, op, "raw=0x{raw:08x}");
        assert_eq!((instr.rd, instr.rn, instr.rm), (2, 5, rm));
        assert_eq!(instr.size, 16);
    }
}
