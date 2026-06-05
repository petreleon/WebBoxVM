use super::scalar_prfm::decode_prfm;
use super::*;

pub(in crate::arm64::decode) fn decode_ldst(raw: u32) -> Option<Instr> {
    let size = (raw >> 30) & 3;
    let opc = (raw >> 22) & 3;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let v = ((raw >> 26) & 1) != 0;
    if v {
        return decode_simd_ldst(raw);
    }
    if size == 3 && opc == 2 {
        return decode_prfm(raw);
    }
    let op = match opc {
        0 => Opcode::Str,
        1 => Opcode::Ldr,
        2 | 3 => Opcode::LdrSign,
        _ => return None,
    };
    let sf = match op {
        Opcode::Str => size == 3,
        Opcode::Ldr => size == 3,
        Opcode::LdrSign => size == 2 || opc == 3,
        _ => unreachable!(),
    };

    let bit24 = (raw >> 24) & 1;
    if bit24 == 1 {
        let imm12 = ((raw >> 10) & 0xFFF) as u64;
        let shift = if size == 3 {
            3
        } else if size == 2 {
            2
        } else {
            size as u8
        };
        return Some(Instr {
            size: 1u8 << size,
            op,
            rd: rt,
            rn,
            rm: 0xFF,
            imm: imm12 << shift,
            sf,
            cond: 0,
        });
    }

    let bit21 = (raw >> 21) & 1;
    let bits11_10 = (raw >> 10) & 3;

    let simm9 = || -> i64 {
        let raw9 = (raw >> 12) & 0x1FF;
        if raw9 & 0x100 != 0 {
            (raw9 as i64) - 0x200
        } else {
            raw9 as i64
        }
    };

    if bit21 == 0 && bits11_10 == 0b00 {
        return Some(Instr {
            size: 1u8 << size,
            op,
            rd: rt,
            rn,
            rm: 0xFF,
            imm: simm9() as u64,
            sf,
            cond: 0,
        });
    }
    if bit21 == 0 && bits11_10 == 0b01 {
        return Some(Instr {
            size: 1u8 << size,
            op,
            rd: rt,
            rn,
            rm: 0xFF,
            imm: simm9() as u64,
            sf,
            cond: 1,
        });
    }
    if bit21 == 0 && bits11_10 == 0b10 {
        return Some(Instr {
            size: 1u8 << size,
            op,
            rd: rt,
            rn,
            rm: 0xFF,
            imm: simm9() as u64,
            sf,
            cond: 0,
        });
    }
    if bit21 == 0 && bits11_10 == 0b11 {
        return Some(Instr {
            size: 1u8 << size,
            op,
            rd: rt,
            rn,
            rm: 0xFF,
            imm: simm9() as u64,
            sf,
            cond: 3,
        });
    }
    if bit21 == 1 && bits11_10 == 2 {
        let rm = ((raw >> 16) & 0x1F) as u8;
        let option = ((raw >> 13) & 7) as u8;
        let s = ((raw >> 12) & 1) as u64;
        return Some(Instr {
            size: 1u8 << size,
            op,
            rd: rt,
            rn,
            rm,
            imm: s,
            sf,
            cond: option,
        });
    }

    None
}
