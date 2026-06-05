use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if raw == 0xD503_251F {
        return DecodeStep::from_option(system::decode_extension_nop(Opcode::Chkfeat, 16));
    }
    if let Some(op) = decode_gcs_alias(raw) {
        return DecodeStep::from_option(system::decode_extension_nop(op, (raw & 0x1F) as u8));
    }
    if (raw & 0xFFF8_0000) == 0xD528_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::Sysl,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: ((raw >> 5) & 0x7FFF) as u64,
            sf: true,
            cond: 0,
            size: 0,
        });
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
