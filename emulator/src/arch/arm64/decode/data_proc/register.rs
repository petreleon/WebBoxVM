use super::*;

pub(in crate::arch::arm64::decode) fn decode_dp_register(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let s = ((raw >> 29) & 1) != 0;
    let shift = ((raw >> 22) & 3) as u8;
    let n = ((raw >> 21) & 1) != 0;
    let rm = ((raw >> 16) & 0x1F) as u8;
    let imm6 = ((raw >> 10) & 0x3F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    if n {
        if shift != 0 {
            return None;
        }
        let option = (imm6 >> 3) & 7;
        let imm3 = imm6 & 7;
        if s && op == 1 && rd == 31 {
            return Some(Instr {
                size: 0,
                op: Opcode::Cmp,
                rd: 31,
                rn,
                rm,
                imm: imm3 as u64,
                sf,
                cond: option | 0x8,
            });
        }
        let opcode = if s {
            if op == 0 {
                Opcode::AddsExt
            } else {
                Opcode::SubsExt
            }
        } else {
            if op == 0 {
                Opcode::AddExt
            } else {
                Opcode::SubExt
            }
        };
        return Some(Instr {
            size: 0,
            op: opcode,
            rd,
            rn,
            rm,
            imm: imm3 as u64,
            sf,
            cond: option,
        });
    }

    if s {
        if op == 1 && rd == 31 {
            return Some(Instr {
                size: 0,
                op: Opcode::Cmp,
                rd: 31,
                rn,
                rm,
                imm: imm6 as u64,
                sf,
                cond: shift,
            });
        }
        let opcode = if op == 0 { Opcode::Adds } else { Opcode::Subs };
        return Some(Instr {
            size: 0,
            op: opcode,
            rd,
            rn,
            rm,
            imm: imm6 as u64,
            sf,
            cond: shift,
        });
    }
    let opcode = if op == 0 { Opcode::Add } else { Opcode::Sub };
    Some(Instr {
        size: 0,
        op: opcode,
        rd,
        rn,
        rm,
        imm: imm6 as u64,
        sf,
        cond: shift,
    })
}

pub(in crate::arch::arm64::decode) fn decode_addsub_carry(raw: u32) -> Option<Instr> {
    if ((raw >> 21) & 0xFF) != 0b11010000 || ((raw >> 10) & 0x3F) != 0 {
        return None;
    }

    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let s = (raw >> 29) & 1;
    let rm = ((raw >> 16) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;
    let opcode = match (op, s) {
        (0, 0) => Opcode::Adc,
        (0, 1) => Opcode::Adcs,
        (1, 0) => Opcode::Sbc,
        (1, 1) => Opcode::Sbcs,
        _ => unreachable!(),
    };
    Some(Instr {
        size: 0,
        op: opcode,
        rd,
        rn,
        rm,
        imm: 0,
        sf,
        cond: 0,
    })
}
