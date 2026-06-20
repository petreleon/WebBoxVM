use super::*;

#[test]
fn decode_adrp_non_zero_immlo() {
    let raw: u32 = 0xf0000d61;
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::Adrp);
    assert_eq!(instr.rd, 1);
    assert_eq!(instr.imm, 0x1af000);
}
