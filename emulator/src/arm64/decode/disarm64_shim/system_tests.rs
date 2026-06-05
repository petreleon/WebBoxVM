use super::*;

#[test]
fn maps_system_mnemonics() {
    let cases = [
        (0xD503_305F, Opcode::NopBarrier, "clrex"),
        (0xD503_20DF, Opcode::NopBarrier, "dgh"),
        (0xD503_30FF, Opcode::NopBarrier, "sb"),
        (0xD503_3BBF, Opcode::NopBarrier, "dmb"),
        (0xD503_3B9F, Opcode::NopBarrier, "dsb"),
        (0xD503_1005, Opcode::Wfe, "wfet"),
        (0xD503_1025, Opcode::Wfi, "wfit"),
        (0xD500_401F, Opcode::Cfinv, "cfinv"),
        (0xBA01_0423, Opcode::Rmif, "rmif"),
        (0x3A00_082D, Opcode::Setf8, "setf8"),
        (0x3A00_482D, Opcode::Setf16, "setf16"),
        (0xD50B_7423, Opcode::DcZva, "sys"),
        (0xD50B_7462, Opcode::DcGva, "sys"),
        (0xD50B_7482, Opcode::DcGzva, "sys"),
        (0xD50B_7520, Opcode::NopBarrier, "sys"),
        (0xD50B_7B22, Opcode::NopBarrier, "sys"),
        (0xD508_871F, Opcode::Tlbi, "sys"),
        (0xD518_4102, Opcode::Msr, "msr"),
        (0xD538_4103, Opcode::Mrs, "mrs"),
        (0xD528_7423, Opcode::Sysl, "sysl"),
        (0xD548_0000, Opcode::Sysp, "sysp"),
        (0xD570_0000, Opcode::Mrrs, "mrrs"),
        (0xD550_0000, Opcode::Msrr, "msrr"),
        (0xD503_42DF, Opcode::Nop, "msr"),
        (0xD400_0001, Opcode::Svc, "svc"),
        (0xD420_0000, Opcode::Brk, "brk"),
        (0x0000_1234, Opcode::Udf, "udf"),
        (0xD69F_03E0, Opcode::Eret, "eret"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode system word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}

#[test]
fn maps_hint_aliases_by_encoding() {
    let cases = [
        (0xD503_201F, Opcode::Nop, "nop"),
        (0xD503_203F, Opcode::Nop, "yield"),
        (0xD503_205F, Opcode::Wfe, "wfe"),
        (0xD503_207F, Opcode::Wfi, "wfi"),
        (0xD503_209F, Opcode::Sev, "sev"),
        (0xD503_20BF, Opcode::Sevl, "sevl"),
        (0xD503_221F, Opcode::Esb, "esb"),
        (0xD503_223F, Opcode::PsbCsync, "psb\t\tcsync"),
        (0xD503_225F, Opcode::TsbCsync, "tsb\t\tcsync"),
        (0xD503_227F, Opcode::GcsbDsync, "gcsb\t\tdsync"),
        (0xD503_229F, Opcode::Csdb, "csdb"),
        (0xD503_22DF, Opcode::Clrbhb, "clrbhb"),
    ];

    for (raw, expected, display) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode system word");
        assert_eq!(format!("{:?}", decoded.mnemonic), "hint");
        assert_eq!(decoded.to_string(), display);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}

#[test]
fn maps_pauth_hint_aliases_by_encoding() {
    for (raw, expected, display) in pauth_hint_cases() {
        let decoded = decoder::decode(raw).expect("disarm64 should decode PAuth hint");
        assert_eq!(format!("{:?}", decoded.mnemonic), "hint");
        assert_eq!(decoded.to_string(), display);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}

#[test]
fn maps_bti_hint_aliases_by_encoding() {
    for (raw, expected, display) in bti_hint_cases() {
        let decoded = decoder::decode(raw).expect("disarm64 should decode BTI hint");
        assert_eq!(format!("{:?}", decoded.mnemonic), "hint");
        assert_eq!(decoded.to_string(), display);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}

#[test]
fn maps_all_barrier_aliases_by_encoding() {
    let raw = 0xD503_3FBF;
    let decoded = decoder::decode(raw).expect("disarm64 should decode dmb sy");

    assert_eq!(format!("{:?}", decoded.mnemonic), "dmb");
    assert_eq!(
        mnemonic_to_opcode(raw, decoded.mnemonic),
        Some(Opcode::NopBarrier)
    );
}

fn pauth_hint_cases() -> [(u32, Opcode, &'static str); 13] {
    [
        (0xD503_211F, Opcode::Pacia1716, "pacia1716"),
        (0xD503_215F, Opcode::Pacib1716, "pacib1716"),
        (0xD503_219F, Opcode::Autia1716, "autia1716"),
        (0xD503_21DF, Opcode::Autib1716, "autib1716"),
        (0xD503_231F, Opcode::Paciaz, "paciaz"),
        (0xD503_233F, Opcode::Paciasp, "paciasp"),
        (0xD503_235F, Opcode::Pacibz, "pacibz"),
        (0xD503_237F, Opcode::Pacibsp, "pacibsp"),
        (0xD503_239F, Opcode::Autiaz, "autiaz"),
        (0xD503_23BF, Opcode::Autiasp, "autiasp"),
        (0xD503_23DF, Opcode::Autibz, "autibz"),
        (0xD503_23FF, Opcode::Autibsp, "autibsp"),
        (0xD503_20FF, Opcode::Xpaclri, "xpaclri"),
    ]
}

fn bti_hint_cases() -> [(u32, Opcode, &'static str); 4] {
    [
        (0xD503_241F, Opcode::Bti, "bti"),
        (0xD503_245F, Opcode::BtiC, "bti\t\tc"),
        (0xD503_249F, Opcode::BtiJ, "bti\t\tj"),
        (0xD503_24DF, Opcode::BtiJc, "bti\t\tjc"),
    ]
}
