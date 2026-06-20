use super::*;

pub(in crate::arch::arm64::decode) fn decode_adr(raw: u32) -> Option<Instr> {
    let op = ((raw >> 31) & 1) != 0;
    let immlo = ((raw >> 29) & 0x3) as i64;
    let immhi = ((raw >> 5) & 0x7FFFF) as i64;
    let mut imm = (immhi << 2) | immlo;
    if imm & (1 << 20) != 0 {
        imm -= 1 << 21;
    }
    let rd = (raw & 0x1F) as u8;
    if op {
        imm <<= 12;
    }
    Some(Instr {
        size: 0,
        op: if op { Opcode::Adrp } else { Opcode::Adr },
        rd,
        rn: 0,
        rm: 0,
        imm: imm as u64,
        sf: true,
        cond: 0,
    })
}

pub(in crate::arch::arm64::decode) fn decode_addsub_imm(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let s = ((raw >> 29) & 1) != 0;
    let sh = ((raw >> 22) & 1) != 0;
    let imm12 = ((raw >> 10) & 0xFFF) as u64;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;
    let imm = if sh { imm12 << 12 } else { imm12 };

    if s {
        if op == 1 && rd == 31 {
            return Some(Instr {
                size: 0,
                op: Opcode::CmpImm,
                rd: 31,
                rn,
                rm: 0,
                imm,
                sf,
                cond: 0,
            });
        }
        let opcode = if op == 0 {
            Opcode::AddsImm
        } else {
            Opcode::SubsImm
        };
        return Some(Instr {
            size: 0,
            op: opcode,
            rd,
            rn,
            rm: 0,
            imm,
            sf,
            cond: 0,
        });
    }
    let opcode = if op == 0 {
        Opcode::AddImm
    } else {
        Opcode::SubImm
    };
    Some(Instr {
        size: 0,
        op: opcode,
        rd,
        rn,
        rm: 0,
        imm,
        sf,
        cond: 0,
    })
}
