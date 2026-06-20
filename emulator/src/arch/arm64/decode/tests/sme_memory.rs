use super::*;

#[test]
fn decode_sme_za_array_memory_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0xE100_0000, Opcode::SmeLdrZa, "ldr"),
        (0xE100_200F, Opcode::SmeLdrZa, "ldr"),
        (0xE100_63EF, Opcode::SmeLdrZa, "ldr"),
        (0xE120_0000, Opcode::SmeStrZa, "str"),
        (0xE120_200F, Opcode::SmeStrZa, "str"),
        (0xE120_63EF, Opcode::SmeStrZa, "str"),
    ]);

    let first = decode(0xE120_0000).unwrap();
    assert_eq!((first.rd, first.rn, first.imm), (12, 0, 0));
    assert_eq!((first.rm, first.cond, first.size), (0xFF, 0, 1));

    let libc_case = decode(0xE120_620F).unwrap();
    assert_eq!((libc_case.rd, libc_case.rn), (15, 16));
    assert_eq!((libc_case.imm, libc_case.cond), (15, 3));

    let libc_load = decode(0xE100_620F).unwrap();
    assert_eq!(
        (libc_load.op, libc_load.rd, libc_load.rn),
        (Opcode::SmeLdrZa, 15, 16)
    );
    assert_eq!((libc_load.imm, libc_load.cond), (15, 3));
}
