use super::*;

#[test]
fn decode_sve_shift_immediate_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x0429_9C20, Opcode::SveLslImm, "lsl"),
        (0x0470_9CA4, Opcode::SveLslImm, "lsl"),
        (0x042F_9528, Opcode::SveLsrImm, "lsr"),
        (0x04A1_95EE, Opcode::SveLsrImm, "lsr"),
        (0x042F_9230, Opcode::SveAsrImm, "asr"),
        (0x04A1_92F6, Opcode::SveAsrImm, "asr"),
        (0x0403_86F9, Opcode::SveLslImm, "lsl"),
        (0x0481_9C3F, Opcode::SveLsrImm, "lsr"),
        (0x0400_81E0, Opcode::SveAsrImm, "asr"),
    ];
    assert_decode_cases(&cases);

    let lsl = decode(0x04FF_9CE6).unwrap(); // lsl z6.d, z7.d, #63
    assert_eq!(
        (lsl.rd, lsl.rn, lsl.cond, lsl.size, lsl.imm),
        (6, 7, 0xFF, 8, 63)
    );

    let pred = decode(0x0403_86F9).unwrap(); // lsl z25.h, p1/m, z25.h, #7
    assert_eq!(
        (pred.rd, pred.rn, pred.cond, pred.size, pred.imm),
        (25, 25, 1, 2, 7)
    );

    assert!(decode(0x0400_8000).is_none());
    assert!(decode(0x0420_9000).is_none());
}
