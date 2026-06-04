use super::*;

#[test]
fn decode_sve_byte_load_store_register_offsets() {
    let cases = [
        (0xA402_4421, Opcode::SveLd1b, "ld1b"),
        (0xA429_4CA7, Opcode::SveLd1b, "ld1b"),
        (0xA444_43E2, Opcode::SveLd1b, "ld1b"),
        (0xA47E_5C1F, Opcode::SveLd1b, "ld1b"),
        (0xE402_4421, Opcode::SveSt1b, "st1b"),
        (0xE429_4CA7, Opcode::SveSt1b, "st1b"),
        (0xE444_43E2, Opcode::SveSt1b, "st1b"),
        (0xE47E_5C1F, Opcode::SveSt1b, "st1b"),
    ];
    assert_decode_cases(&cases);

    let load = decode(0xA429_4CA7).unwrap(); // ld1b { z7.h }, p3/z, [x5, x9]
    assert_eq!(load.rd, 7);
    assert_eq!(load.rn, 5);
    assert_eq!(load.rm, 9);
    assert_eq!(load.cond, 3);
    assert_eq!(load.size, 2);
    assert_eq!(load.imm, 0);

    let store = decode(0xE47E_5C1F).unwrap(); // st1b { z31.d }, p7, [x0, x30]
    assert_eq!(store.rd, 31);
    assert_eq!(store.rn, 0);
    assert_eq!(store.rm, 30);
    assert_eq!(store.cond, 7);
    assert_eq!(store.size, 8);

    assert!(decode(0xA41F_4421).is_none());
    assert!(decode(0xE41F_4421).is_none());
}
