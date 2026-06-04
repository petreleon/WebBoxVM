use super::*;

pub(in crate::arm64::decode) fn decode_ldst_excl(raw: u32) -> Option<Instr> {
    let size = (raw >> 30) & 3;
    let l = (raw >> 22) & 1;
    let o1 = (raw >> 23) & 1;
    let o0 = (raw >> 15) & 1;
    let pair = ((raw >> 21) & 1) != 0;
    let rs = ((raw >> 16) & 0x1F) as u8;
    let rt2 = ((raw >> 10) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;

    if pair {
        let sf = size == 3;
        return if l == 1 {
            Some(Instr {
                op: Opcode::Ldxp,
                rd: rt,
                rn,
                rm: rt2,
                imm: 0,
                sf,
                cond: o0 as u8,
                size: 0,
            })
        } else {
            Some(Instr {
                op: Opcode::Stxp,
                rd: rt,
                rn,
                rm: rt2,
                imm: rs as u64,
                sf,
                cond: o0 as u8,
                size: 0,
            })
        };
    }

    if l == 1 {
        let op = if o1 == 1 && o0 == 1 && rs == 31 && rt2 == 31 {
            Opcode::Ldar
        } else {
            Opcode::Ldxr
        };
        let sz_bytes = 1 << size;
        Some(Instr {
            op,
            rd: rt,
            rn,
            rm: rt2,
            imm: 0,
            sf: size == 3,
            cond: o0 as u8,
            size: sz_bytes,
        })
    } else {
        let op = if o1 == 1 && o0 == 1 && rt2 == 31 && rs == 31 {
            Opcode::Stlr
        } else {
            Opcode::Stxr
        };
        let sz_bytes = 1 << size;
        Some(Instr {
            op,
            rd: rt,
            rn,
            rm: rt2,
            imm: rs as u64,
            sf: size == 3,
            cond: o0 as u8,
            size: sz_bytes,
        })
    }
}
