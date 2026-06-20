use super::*;

#[test]
fn decodes_simd_rbit_vector_forms() {
    let cases = [
        (0x2E60_5800, 0, 0, 8),
        (0x6E60_5908, 8, 8, 16),
        (0x6E60_5922, 2, 9, 16),
    ];

    for (raw, rd, rn, size) in cases {
        assert_disarm64_mnemonic(raw, "rbit");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::SimdRbit);
        assert_eq!((instr.rd, instr.rn, instr.size), (rd, rn, size));
    }
}
