use super::*;

pub(in crate::arm64::decode) fn decode_lse_atomic(raw: u32) -> Option<Instr> {
    let size_field = ((raw >> 30) & 3) as u8;
    let size = 1u8 << size_field;
    let rs = ((raw >> 16) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let sf = size == 8;

    if (raw & 0x3F20_7C00) == 0x0820_7C00 {
        if size_field <= 1 {
            let elem_size = if size_field == 0 { 4 } else { 8 };
            return Some(Instr {
                size: elem_size,
                op: Opcode::Casp,
                rd: rs,
                rn,
                rm: rt,
                imm: 0,
                sf: elem_size == 8,
                cond: 0,
            });
        }
        return Some(Instr {
            size,
            op: Opcode::Cas,
            rd: rs,
            rn,
            rm: rt,
            imm: 0,
            sf,
            cond: 0,
        });
    }

    if (raw & 0xFF20_0C00) == 0x1920_0000 {
        let atomic_op = ((raw >> 12) & 0xF) as u64;
        if !matches!(atomic_op, 0x1 | 0x3 | 0x8) {
            return None;
        }
        if rt == 31 || rs == 31 || rt == rs {
            return None;
        }
        return Some(Instr {
            size: 8,
            op: Opcode::AtomicPair,
            rd: rt,
            rn,
            rm: rs,
            imm: atomic_op,
            sf: true,
            cond: 0,
        });
    }

    if (raw & 0x3F20_0C00) == 0x3820_0000 {
        let atomic_op = ((raw >> 12) & 0xF) as u64;
        return Some(Instr {
            size,
            op: Opcode::Atomic,
            rd: rt,
            rn,
            rm: rs,
            imm: atomic_op,
            sf,
            cond: 0,
        });
    }

    None
}
