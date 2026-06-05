use super::*;

#[test]
fn movz_lsl_0() {
    let instr = decode(0xD282_4680).unwrap();
    assert_eq!(instr.op, Opcode::Movz);
    assert_eq!(instr.imm, 0x1234);
}

#[test]
fn movz_lsl_16() {
    let instr = decode(0xD2A2_4680).unwrap();
    assert_eq!(instr.imm, 0x1234_0000);
}

#[test]
fn decode_cmp_x3_x2() {
    let instr = decode(0xEB02007F).unwrap();
    assert_eq!(instr.op, Opcode::Cmp);
    assert_eq!(instr.rn, 3);
    assert_eq!(instr.rm, 2);
}

#[test]
fn decode_addsub_with_carry() {
    let adc = decode(0x9A02_0020).unwrap(); // adc x0, x1, x2
    assert_eq!(adc.op, Opcode::Adc);
    assert_eq!(adc.rd, 0);
    assert_eq!(adc.rn, 1);
    assert_eq!(adc.rm, 2);

    let sbc = decode(0xDA1F_03E0).unwrap(); // sbc x0, xzr, xzr
    assert_eq!(sbc.op, Opcode::Sbc);
    assert_eq!(sbc.rd, 0);
    assert_eq!(sbc.rn, 31);
    assert_eq!(sbc.rm, 31);
}

#[test]
fn decode_crc32_scalar_forms() {
    let cases = [
        (0x1AC5_4042, Opcode::Crc32, 1, "crc32b"),
        (0x1AC5_4442, Opcode::Crc32, 2, "crc32h"),
        (0x1AC3_4842, Opcode::Crc32, 4, "crc32w"),
        (0x9AC4_4C42, Opcode::Crc32, 8, "crc32x"),
        (0x1AC5_5042, Opcode::Crc32c, 1, "crc32cb"),
        (0x1AC5_5442, Opcode::Crc32c, 2, "crc32ch"),
        (0x1AC3_5842, Opcode::Crc32c, 4, "crc32cw"),
        (0x9AC4_5C42, Opcode::Crc32c, 8, "crc32cx"),
    ];

    for (raw, expected, size, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.rd, 2);
        assert_eq!(instr.rn, 2);
        assert_eq!(instr.size, size);
    }

    assert_eq!(decode(0x1AC5_4042).unwrap().rm, 5);
    assert_eq!(decode(0x1AC3_4842).unwrap().rm, 3);
    assert_eq!(decode(0x9AC4_4C42).unwrap().rm, 4);
}

#[test]
fn decode_pointer_subtract_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x9AC0_0041, Opcode::Subp, "subp", 1, 2, 0),
        (0xBAC3_0062, Opcode::Subps, "subps", 2, 3, 3),
    ];
    for (raw, expected, mnemonic, rd, rn, rm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!((instr.rd, instr.rn, instr.rm, instr.sf), (rd, rn, rm, true));
    }
}

#[test]
fn decode_cls_scalar_forms() {
    let cases = [
        (0x5AC0_1420, false, "cls"),
        (0xDAC0_1420, true, "cls"),
    ];

    for (raw, sf, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::Cls);
        assert_eq!((instr.rd, instr.rn, instr.sf), (0, 1, sf));
    }
}

#[test]
fn decode_logical_immediate_forms() {
    let cases = [
        (0x9240_1C20, Opcode::AndImm, "and", 0xff),
        (0xB278_1C62, Opcode::OrrImm, "orr", 0xff00),
        (0x5200_9CA4, Opcode::EorImm, "eor", 0x00ff_00ff),
        (0xF204_CCE6, Opcode::AndsImm, "ands", 0xf0f0_f0f0_f0f0_f0f0),
    ];

    for (raw, expected, mnemonic, imm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, imm, "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_addsub_immediate_forms() {
    let cases = [
        (0x9104_8C20, Opcode::AddImm, "add", 0x123),
        (0x9140_1062, Opcode::AddImm, "add", 0x4000),
        (0xD101_54A4, Opcode::SubImm, "sub", 0x55),
        (0x3100_40E6, Opcode::AddsImm, "adds", 0x10),
        (0xF100_896A, Opcode::SubsImm, "subs", 0x22),
        (0xF100_811F, Opcode::CmpImm, "subs", 0x20),
    ];

    for (raw, expected, mnemonic, imm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, imm, "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_addsub_shifted_rd31_cross_checked_with_disarm64() {
    let cases = [(0x8B02_003F, Opcode::Add, "add"), (0xCB02_003F, Opcode::Sub, "sub")];
    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.rd, 31);
    }
}

#[test]
fn decode_addsub_extended_forms() {
    let cases = [
        (0x8B22_4820, Opcode::AddExt, "add", 2, 2),
        (0xCB25_E483, Opcode::SubExt, "sub", 7, 1),
        (0x2B28_0CE6, Opcode::AddsExt, "adds", 0, 3),
        (0xEB2B_A949, Opcode::SubsExt, "subs", 5, 2),
        (0xEB2D_499F, Opcode::Cmp, "subs", 10, 2),
    ];

    for (raw, expected, mnemonic, cond, imm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.cond, cond, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, imm, "raw=0x{raw:08x}");
    }
}
