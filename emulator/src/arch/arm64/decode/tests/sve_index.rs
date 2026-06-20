use super::*;

#[test]
fn decode_sve_index_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x04A3_43C4, Opcode::SveIndex, "index"),
        (0x04A7_4B82, Opcode::SveIndex, "index"),
        (0x04BE_4690, Opcode::SveIndex, "index"),
        (0x04AA_4D3F, Opcode::SveIndex, "index"),
    ];
    assert_decode_cases(&cases);

    let rr = decode(0x04AA_4D3F).unwrap(); // index z31.s, w9, w10
    assert_eq!((rr.rd, rr.rn, rr.rm, rr.size, rr.cond), (31, 9, 10, 4, 0));

    let ri = decode(0x04BE_4690).unwrap(); // index z16.s, w20, #-2
    assert_eq!((ri.rd, ri.rn, ri.size, ri.cond), (16, 20, 4, 2));
    assert_eq!((ri.imm >> 32) as u32 as i32, -2);

    let ii = decode(0x04A3_43C4).unwrap(); // index z4.s, #-2, #3
    assert_eq!((ii.rd, ii.size, ii.cond), (4, 4, 3));
    assert_eq!(
        (ii.imm as u32 as i32, (ii.imm >> 32) as u32 as i32),
        (-2, 3)
    );
}
