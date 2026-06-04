use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_unpredicated(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_predicated(raw) {
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}

fn decode_unpredicated(raw: u32) -> Option<Instr> {
    if (raw & 0xFF20_F000) != 0x0420_9000 {
        return None;
    }
    let op = op_from_bits((raw >> 10) & 0x3)?;
    let tsize = (((raw >> 22) & 0x3) << 2) | ((raw >> 19) & 0x3);
    build(raw, op, tsize, (raw >> 16) & 0x7, (raw >> 5) & 0x1F, 0xFF)
}

fn decode_predicated(raw: u32) -> Option<Instr> {
    if (raw & 0xFF3C_E000) != 0x0400_8000 {
        return None;
    }
    let op = op_from_bits((raw >> 16) & 0x3)?;
    let tsize = (((raw >> 22) & 0x3) << 2) | ((raw >> 8) & 0x3);
    build(
        raw,
        op,
        tsize,
        (raw >> 5) & 0x7,
        raw & 0x1F,
        ((raw >> 10) & 0x7) as u8,
    )
}

fn op_from_bits(bits: u32) -> Option<Opcode> {
    match bits {
        0 => Some(Opcode::SveAsrImm),
        1 => Some(Opcode::SveLsrImm),
        3 => Some(Opcode::SveLslImm),
        _ => None,
    }
}

fn build(raw: u32, op: Opcode, tsize: u32, imm3: u32, rn: u32, cond: u8) -> Option<Instr> {
    let size = element_size(tsize)?;
    let esize = (size as u64) * 8;
    let encoded = ((tsize << 3) | imm3) as u64;
    let shift = if op == Opcode::SveLslImm {
        encoded.checked_sub(esize)?
    } else {
        (2 * esize).checked_sub(encoded)?
    };
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: rn as u8,
        rm: 0xFF,
        imm: shift,
        sf: false,
        cond,
        size,
    })
}

fn element_size(tsize: u32) -> Option<u8> {
    if tsize == 0 {
        return None;
    }
    let high = 31 - tsize.leading_zeros();
    Some(1u8 << high)
}
