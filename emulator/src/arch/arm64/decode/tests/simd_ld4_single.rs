use super::*;

#[test]
fn decodes_simd_ld4_single_lane_forms() {
    let cases = [
        (0x0D60_A004, 4, 0, 0xFF, 0),
        (0x0DFF_A004, 4, 0, 0xFE, 0),
        (0x0DFF_B144, 4, 10, 0xFE, 1),
        (0x4D60_A144, 4, 10, 0xFF, 2),
        (0x0DE4_B004, 4, 0, 4, 1),
    ];

    for (raw, rd, rn, rm, lane) in cases {
        assert_disarm64_mnemonic(raw, "ld4");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::SimdLd4Single, "raw=0x{raw:08x}");
        assert_eq!((instr.rd, instr.rn, instr.rm), (rd, rn, rm));
        assert_eq!((instr.imm, instr.cond, instr.size), (lane, 4, 4));
    }
}
