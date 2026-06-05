use super::*;

mod pauth_register;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if raw == 0xD503_251F {
        return DecodeStep::from_option(system::decode_extension_nop(Opcode::Chkfeat, 16));
    }
    if let Some(op) = decode_event_hint(raw) {
        return DecodeStep::from_option(system::decode_extension_nop(op, 0));
    }
    if let Some(op) = decode_basic_hint(raw) {
        return DecodeStep::from_option(system::decode_extension_nop(op, 0));
    }
    if let Some(op) = decode_sync_hint(raw) {
        return DecodeStep::from_option(system::decode_extension_nop(op, 0));
    }
    if let Some(op) = decode_bti_hint(raw) {
        return DecodeStep::from_option(system::decode_extension_nop(op, 0));
    }
    if let Some(op) = decode_pauth_hint(raw) {
        return DecodeStep::from_option(system::decode_extension_nop(op, 0));
    }
    if let Some(instr) = pauth_register::decode(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(op) = decode_gcs_alias(raw) {
        return DecodeStep::from_option(system::decode_extension_nop(op, (raw & 0x1F) as u8));
    }
    if (raw & 0xFFF8_0000) == 0xD528_0000 {
        return DecodeStep::Hit(system_instr(raw, Opcode::Sysl));
    }
    if (raw & 0xFFF8_0000) == 0xD548_0000 {
        return DecodeStep::Hit(system_instr(raw, Opcode::Sysp));
    }
    if (raw & 0xFFF0_0000) == 0xD570_0000 {
        return DecodeStep::Hit(system_instr(raw, Opcode::Mrrs));
    }
    if (raw & 0xFFF0_0000) == 0xD550_0000 {
        return DecodeStep::Hit(system_instr(raw, Opcode::Msrr));
    }
    if matches!(raw, 0xD503_427F | 0xD503_447F | 0xD503_467F) {
        return DecodeStep::from_option(system::decode_extension_nop(Opcode::Smstop, 0));
    }
    match raw & 0xFFFF_FFE0 {
        0xD50B_7460 => DecodeStep::from_option(system::decode_extension_nop(
            Opcode::DcGva,
            (raw & 0x1F) as u8,
        )),
        0xD50B_7480 => DecodeStep::from_option(system::decode_extension_nop(
            Opcode::DcGzva,
            (raw & 0x1F) as u8,
        )),
        _ => DecodeStep::Miss,
    }
}

fn decode_event_hint(raw: u32) -> Option<Opcode> {
    Some(match raw {
        0xD503_209F => Opcode::Sev,
        0xD503_20BF => Opcode::Sevl,
        _ => return None,
    })
}

fn decode_basic_hint(raw: u32) -> Option<Opcode> {
    Some(match raw {
        0xD503_20DF => Opcode::Dgh,
        0xD503_30FF => Opcode::Sb,
        _ => return None,
    })
}

fn decode_sync_hint(raw: u32) -> Option<Opcode> {
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

fn decode_bti_hint(raw: u32) -> Option<Opcode> {
    Some(match raw {
        0xD503_241F => Opcode::Bti,
        0xD503_245F => Opcode::BtiC,
        0xD503_249F => Opcode::BtiJ,
        0xD503_24DF => Opcode::BtiJc,
        _ => return None,
    })
}

fn decode_pauth_hint(raw: u32) -> Option<Opcode> {
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

fn system_instr(raw: u32, op: Opcode) -> Instr {
    Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: 0,
        rm: 0,
        imm: ((raw >> 5) & 0x7FFF) as u64,
        sf: true,
        cond: 0,
        size: 0,
    }
}

fn decode_gcs_alias(raw: u32) -> Option<Opcode> {
    match raw {
        0xD508_779F => return Some(Opcode::GcsPushX),
        0xD508_77BF => return Some(Opcode::GcsPopCx),
        0xD508_77DF => return Some(Opcode::GcsPopX),
        _ => {}
    }
    match raw & 0xFFFF_FFE0 {
        0xD50B_7700 => Some(Opcode::GcsPushM),
        0xD52B_7720 => Some(Opcode::GcsPopM),
        0xD50B_7740 => Some(Opcode::GcsSs1),
        0xD52B_7760 => Some(Opcode::GcsSs2),
        _ => None,
    }
}
