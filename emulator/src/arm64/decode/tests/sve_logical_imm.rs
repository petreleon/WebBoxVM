use super::*;

#[test]
fn decode_sve_logical_immediate_forms() {
    let cases = [
        (0x0582_00E0, Opcode::SveAndImm, "and"),
        (0x0500_04E1, Opcode::SveOrrImm, "orr"),
        (0x0540_44E2, Opcode::SveEorImm, "eor"),
        (0x05C2_00E3, Opcode::SveDupm, "dupm"),
        (0x0582_00C6, Opcode::SveAndImm, "and"),
        (0x0502_6122, Opcode::SveOrrImm, "orr"),
        (0x05C2_57DD, Opcode::SveDupm, "dupm"),
    ];

    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let and = decode(0x0582_00E0).unwrap(); // and z0.d, z0.d, #0xff
    assert_eq!(and.rd, 0);
    assert_eq!(and.rn, 0);
    assert_eq!(and.imm, 0xff);
    assert_eq!(and.size, 8);

    let orr = decode(0x0500_04E1).unwrap(); // orr z1.h, z1.h, #0xff
    assert_eq!(orr.rd, 1);
    assert_eq!(orr.imm, 0x00ff_00ff_00ff_00ff);

    let eor = decode(0x0540_44E2).unwrap(); // eor z2.h, z2.h, #0xff00
    assert_eq!(eor.rd, 2);
    assert_eq!(eor.imm, 0xff00_ff00_ff00_ff00);

    let dupm = decode(0x05C2_00E3).unwrap(); // dupm z3.d, #0xff
    assert_eq!(dupm.rd, 3);
    assert_eq!(dupm.imm, 0xff);
}
