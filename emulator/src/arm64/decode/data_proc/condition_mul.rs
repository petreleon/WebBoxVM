use super::*;

pub(in crate::arm64::decode) fn decode_condsel(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let o2 = ((raw >> 10) & 1) != 0;
    let cond = ((raw >> 12) & 0xF) as u8;
    let o3 = ((raw >> 11) & 1) != 0;
    let _rm = ((raw >> 16) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;
    let bits22_21 = (raw >> 21) & 3;

    if bits22_21 == 0 {
        let opcode = match (op, o2, o3) {
            (0, false, false) => Opcode::Csel,
            (0, true, false) => Opcode::Csinc,
            (1, false, false) => Opcode::Csinv,
            (1, true, false) => Opcode::Csneg,
            _ => return None,
        };
        return Some(Instr {
            size: 0,
            op: opcode,
            rd,
            rn,
            rm: _rm,
            imm: 0,
            sf,
            cond,
        });
    }

    if bits22_21 != 2 {
        return None;
    }
    let is_immediate = ((raw >> 10) & 0x3) == 0b10;
    if !is_immediate && ((raw >> 10) & 0x3) != 0 {
        return None;
    }
    let nzcv = (raw & 0xF) as u64;
    let rm_or_imm = ((raw >> 16) & 0x1F) as u64;
    let opcode = if op == 0 { Opcode::Ccmn } else { Opcode::Ccmp };
    Some(Instr {
        size: is_immediate as u8,
        op: opcode,
        rd,
        rn,
        rm: rm_or_imm as u8,
        imm: nzcv,
        sf,
        cond,
    })
}

pub(in crate::arm64::decode) fn decode_mul(raw: u32) -> Option<Instr> {
    let bits31_29 = (raw >> 29) & 0x7;
    let op54 = (raw >> 21) & 0x7;
    let o0 = ((raw >> 15) & 1) != 0;
    let rd = (raw & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let ra = ((raw >> 10) & 0x1F) as u8;
    let rm = ((raw >> 16) & 0x1F) as u8;

    let (sf, size) = match bits31_29 {
        0b000 => {
            if op54 == 0b000 {
                (false, 0)
            } else {
                return None;
            }
        }
        0b100 => match op54 {
            0b000 => (true, 0),
            0b001 => (true, 2),
            0b101 => (true, 1),
            0b010 => {
                return Some(Instr {
                    op: Opcode::Smulh,
                    rd,
                    rn,
                    rm,
                    imm: 0,
                    sf: true,
                    cond: 0,
                    size: 0,
                });
            }
            0b110 => {
                return Some(Instr {
                    op: Opcode::Umulh,
                    rd,
                    rn,
                    rm,
                    imm: 0,
                    sf: true,
                    cond: 0,
                    size: 0,
                });
            }
            _ => return None,
        },
        _ => return None,
    };

    let op = if o0 { Opcode::Msub } else { Opcode::Madd };
    Some(Instr {
        op,
        rd,
        rn,
        rm,
        imm: 0,
        sf,
        cond: ra,
        size,
    })
}
