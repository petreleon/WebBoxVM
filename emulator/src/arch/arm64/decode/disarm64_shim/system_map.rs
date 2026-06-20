use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#hint if raw == 0xD503_203F => Opcode::Yield,
        M::r#hint if raw == 0xD503_205F => Opcode::Wfe,
        M::r#hint if raw == 0xD503_207F => Opcode::Wfi,
        M::r#hint => event_hint(raw)
            .or_else(|| sync_hint(raw))
            .or_else(|| bti_hint(raw))
            .or_else(|| pauth_hint(raw))
            .unwrap_or(Opcode::Nop),
        M::r#cfinv if raw == 0xD500_401F => Opcode::Cfinv,
        M::r#chkfeat if raw == 0xD503_251F => Opcode::Chkfeat,
        M::r#clrex if raw == 0xD503_305F => Opcode::Clrex,
        M::r#dgh if raw == 0xD503_20DF => Opcode::Dgh,
        M::r#sb if raw == 0xD503_30FF => Opcode::Sb,
        M::r#dmb if dmb(raw) => Opcode::Dmb,
        M::r#dsb if dsb(raw) => Opcode::Dsb,
        M::r#isb if isb(raw) => Opcode::Isb,
        M::r#pacia if (raw & 0xFFFF_FC00) == 0xDAC1_0000 => Opcode::Pacia,
        M::r#pacib if (raw & 0xFFFF_FC00) == 0xDAC1_0400 => Opcode::Pacib,
        M::r#pacda if (raw & 0xFFFF_FC00) == 0xDAC1_0800 => Opcode::Pacda,
        M::r#pacdb if (raw & 0xFFFF_FC00) == 0xDAC1_0C00 => Opcode::Pacdb,
        M::r#autia if (raw & 0xFFFF_FC00) == 0xDAC1_1000 => Opcode::Autia,
        M::r#autib if (raw & 0xFFFF_FC00) == 0xDAC1_1400 => Opcode::Autib,
        M::r#autda if (raw & 0xFFFF_FC00) == 0xDAC1_1800 => Opcode::Autda,
        M::r#autdb if (raw & 0xFFFF_FC00) == 0xDAC1_1C00 => Opcode::Autdb,
        M::r#paciza if (raw & 0xFFFF_FFE0) == 0xDAC1_23E0 => Opcode::Paciza,
        M::r#pacizb if (raw & 0xFFFF_FFE0) == 0xDAC1_27E0 => Opcode::Pacizb,
        M::r#pacdza if (raw & 0xFFFF_FFE0) == 0xDAC1_2BE0 => Opcode::Pacdza,
        M::r#pacdzb if (raw & 0xFFFF_FFE0) == 0xDAC1_2FE0 => Opcode::Pacdzb,
        M::r#autiza if (raw & 0xFFFF_FFE0) == 0xDAC1_33E0 => Opcode::Autiza,
        M::r#autizb if (raw & 0xFFFF_FFE0) == 0xDAC1_37E0 => Opcode::Autizb,
        M::r#autdza if (raw & 0xFFFF_FFE0) == 0xDAC1_3BE0 => Opcode::Autdza,
        M::r#autdzb if (raw & 0xFFFF_FFE0) == 0xDAC1_3FE0 => Opcode::Autdzb,
        M::r#xpaci if (raw & 0xFFFF_FFE0) == 0xDAC1_43E0 => Opcode::Xpaci,
        M::r#xpacd if (raw & 0xFFFF_FFE0) == 0xDAC1_47E0 => Opcode::Xpacd,
        M::r#wfet if (raw & 0xFFFF_FFE0) == 0xD503_1000 => Opcode::Wfe,
        M::r#wfit if (raw & 0xFFFF_FFE0) == 0xD503_1020 => Opcode::Wfi,
        M::r#rmif if (raw & 0xFFE0_7C10) == 0xBA00_0400 => Opcode::Rmif,
        M::r#setf8 if (raw & 0xFFFF_FC1F) == 0x3A00_080D => Opcode::Setf8,
        M::r#setf16 if (raw & 0xFFFF_FC1F) == 0x3A00_480D => Opcode::Setf16,
        M::r#gcspushm if (raw & 0xFFFF_FFE0) == 0xD50B_7700 => Opcode::GcsPushM,
        M::r#gcspushx if raw == 0xD508_779F => Opcode::GcsPushX,
        M::r#gcspopm if (raw & 0xFFFF_FFE0) == 0xD52B_7720 => Opcode::GcsPopM,
        M::r#gcspopx if raw == 0xD508_77DF => Opcode::GcsPopX,
        M::r#gcspopcx if raw == 0xD508_77BF => Opcode::GcsPopCx,
        M::r#gcsss1 if (raw & 0xFFFF_FFE0) == 0xD50B_7740 => Opcode::GcsSs1,
        M::r#gcsss2 if (raw & 0xFFFF_FFE0) == 0xD52B_7760 => Opcode::GcsSs2,
        M::r#smstop if smstop(raw) => Opcode::Smstop,
        M::r#mrs if ((raw >> 20) & 0xFFF) == 0xD53 => Opcode::Mrs,
        M::r#msr if ((raw >> 20) & 0xFFF) == 0xD51 => Opcode::Msr,
        M::r#msr if daif_set(raw) => Opcode::DaifSet,
        M::r#msr if daif_clr(raw) => Opcode::DaifClr,
        M::r#sysl if (raw & 0xFFF8_0000) == 0xD528_0000 => Opcode::Sysl,
        M::r#sysp if (raw & 0xFFF8_0000) == 0xD548_0000 => Opcode::Sysp,
        M::r#mrrs if (raw & 0xFFF0_0000) == 0xD570_0000 => Opcode::Mrrs,
        M::r#msrr if (raw & 0xFFF0_0000) == 0xD550_0000 => Opcode::Msrr,
        M::r#sys if (raw & 0xFFFF_FFE0) == 0xD50B_7420 => Opcode::DcZva,
        M::r#sys if (raw & 0xFFFF_FFE0) == 0xD50B_7460 => Opcode::DcGva,
        M::r#sys if (raw & 0xFFFF_FFE0) == 0xD50B_7480 => Opcode::DcGzva,
        M::r#sys if cache_maintenance(raw) => Opcode::NopBarrier,
        M::r#sys if legacy_tlbi(raw) => Opcode::Tlbi,
        M::r#svc => Opcode::Svc,
        M::r#brk => Opcode::Brk,
        M::r#udf => Opcode::Udf,
        M::r#eret => Opcode::Eret,
        _ => return None,
    })
}

fn dmb(raw: u32) -> bool {
    (raw & 0xFFFF_F0FF) == 0xD503_30BF
}

fn dsb(raw: u32) -> bool {
    (raw & 0xFFFF_F0FF) == 0xD503_309F || (raw & 0xFFFF_F3FF) == 0xD503_323F
}

fn isb(raw: u32) -> bool {
    (raw & 0xFFFF_F0FF) == 0xD503_30DF
}

fn cache_maintenance(raw: u32) -> bool {
    matches!(raw & 0xFFFF_FFE0, 0xD50B_7520 | 0xD50B_7B20)
}

fn daif_set(raw: u32) -> bool {
    daif_alias(raw, 0b110)
}

fn daif_clr(raw: u32) -> bool {
    daif_alias(raw, 0b111)
}

fn daif_alias(raw: u32, op2: u32) -> bool {
    (raw & 0xFFFF_F01F) == 0xD503_401F && ((raw >> 5) & 0x7) == op2
}

fn legacy_tlbi(raw: u32) -> bool {
    let op0 = (raw >> 19) & 0x3;
    let l = (raw >> 21) & 1;
    let crn = (raw >> 12) & 0xF;
    l == 0 && op0 == 1 && crn == 8
}

fn smstop(raw: u32) -> bool {
    matches!(raw, 0xD503_427F | 0xD503_447F | 0xD503_467F)
}

fn event_hint(raw: u32) -> Option<Opcode> {
    Some(match raw {
        0xD503_209F => Opcode::Sev,
        0xD503_20BF => Opcode::Sevl,
        _ => return None,
    })
}

fn sync_hint(raw: u32) -> Option<Opcode> {
    Some(match raw {
        0xD503_221F => Opcode::Esb,
        0xD503_223F => Opcode::PsbCsync,
        0xD503_225F => Opcode::TsbCsync,
        0xD503_227F => Opcode::GcsbDsync,
        0xD503_229F => Opcode::Csdb,
        0xD503_22DF => Opcode::Clrbhb,
        _ => return None,
    })
}

fn bti_hint(raw: u32) -> Option<Opcode> {
    Some(match raw {
        0xD503_241F => Opcode::Bti,
        0xD503_245F => Opcode::BtiC,
        0xD503_249F => Opcode::BtiJ,
        0xD503_24DF => Opcode::BtiJc,
        _ => return None,
    })
}

fn pauth_hint(raw: u32) -> Option<Opcode> {
    Some(match raw {
        0xD503_211F => Opcode::Pacia1716,
        0xD503_215F => Opcode::Pacib1716,
        0xD503_219F => Opcode::Autia1716,
        0xD503_21DF => Opcode::Autib1716,
        0xD503_231F => Opcode::Paciaz,
        0xD503_233F => Opcode::Paciasp,
        0xD503_235F => Opcode::Pacibz,
        0xD503_237F => Opcode::Pacibsp,
        0xD503_239F => Opcode::Autiaz,
        0xD503_23BF => Opcode::Autiasp,
        0xD503_23DF => Opcode::Autibz,
        0xD503_23FF => Opcode::Autibsp,
        0xD503_20FF => Opcode::Xpaclri,
        _ => return None,
    })
}
