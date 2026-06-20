use super::*;

pub(in crate::arch::arm64::decode) fn decode_bitfield(raw: u32) -> Option<Instr> {
    let opc = ((raw >> 29) & 3) as u8;
    let sf = ((raw >> 31) & 1) != 0;
    let n = ((raw >> 22) & 1) != 0;
    if n != sf {
        return None;
    }
    let immr = ((raw >> 16) & 0x3F) as u8;
    let imms = ((raw >> 10) & 0x3F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;
    let width = if sf { 64 } else { 32 };
    if immr >= width || imms >= width {
        return None;
    }

    if opc == 0 && immr == 0 && imms == 31 {
        return Some(Instr {
            size: 0,
            op: Opcode::Sxtw,
            rd,
            rn,
            rm: 0,
            imm: 32,
            sf,
            cond: 0,
        });
    }

    let op = match opc {
        0 => Opcode::Sbfm,
        1 => Opcode::Bfm,
        2 => Opcode::Ubfm,
        _ => return None,
    };

    Some(Instr {
        size: 0,
        op,
        rd,
        rn,
        rm: immr,
        imm: imms as u64,
        sf,
        cond: 0,
    })
}

pub(in crate::arch::arm64::decode) fn decode_extract(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let n = ((raw >> 22) & 1) != 0;
    if n != sf {
        return None;
    }

    let rm = ((raw >> 16) & 0x1F) as u8;
    let imm = ((raw >> 10) & 0x3F) as u64;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;
    if !sf && imm >= 32 {
        return None;
    }

    Some(Instr {
        size: 0,
        op: Opcode::Extr,
        rd,
        rn,
        rm,
        imm,
        sf,
        cond: 0,
    })
}
