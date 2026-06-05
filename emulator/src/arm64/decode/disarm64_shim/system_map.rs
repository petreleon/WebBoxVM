use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#hint if raw == 0xD503_205F => Opcode::Wfe,
        M::r#hint if raw == 0xD503_207F => Opcode::Wfi,
        M::r#hint => event_hint(raw)
            .or_else(|| bti_hint(raw))
            .or_else(|| pauth_hint(raw))
            .unwrap_or(Opcode::Nop),
        M::r#cfinv if raw == 0xD500_401F => Opcode::Cfinv,
        M::r#chkfeat if raw == 0xD503_251F => Opcode::Chkfeat,
        M::r#clrex if raw == 0xD503_305F => Opcode::NopBarrier,
        M::r#dgh if raw == 0xD503_20DF => Opcode::NopBarrier,
        M::r#sb if raw == 0xD503_30FF => Opcode::NopBarrier,
        M::r#dmb | M::r#dsb | M::r#isb if barrier(raw) => Opcode::NopBarrier,
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
        M::r#msr if legacy_daif_alias(raw) => Opcode::Nop,
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

fn barrier(raw: u32) -> bool {
    matches!(raw & 0xFFFF_F0FF, 0xD503_309F | 0xD503_30BF | 0xD503_30DF)
}

fn cache_maintenance(raw: u32) -> bool {
    matches!(raw & 0xFFFF_FFE0, 0xD50B_7520 | 0xD50B_7B20)
}

fn legacy_daif_alias(raw: u32) -> bool {
    (raw & 0xFFFF_F01F) == 0xD503_401F && matches!((raw >> 5) & 0x7, 0b110 | 0b111)
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
