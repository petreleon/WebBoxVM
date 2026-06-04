use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBF20_FC00) == 0x0E00_1800 {
        return decode_perm(raw, Opcode::SimdUzp1);
    }
    if (raw & 0xBF20_FC00) == 0x0E00_5800 {
        return decode_perm(raw, Opcode::SimdUzp2);
    }
    if (raw & 0xBF20_FC00) == 0x0E00_2800 {
        return decode_perm(raw, Opcode::SimdTrn1);
    }
    if (raw & 0xBF20_FC00) == 0x0E00_6800 {
        return decode_perm(raw, Opcode::SimdTrn2);
    }
    if (raw & 0xBF20_FC00) == 0x0E00_3800 {
        return decode_perm(raw, Opcode::SimdZip1);
    }
    if (raw & 0xBF20_FC00) == 0x0E00_7800 {
        return decode_perm(raw, Opcode::SimdZip2);
    }
    if (raw & 0xBFE0_9C00) == 0x0E00_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdTbl,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: (((raw >> 13) & 0x3) + 1) as u8,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFFF_FC00) == 0x4E28_4800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAese,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x4E28_5800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAesd,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x4E28_6800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAesmc,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x4E28_7800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAesimc,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    DecodeStep::Miss
}

fn decode_perm(raw: u32, op: Opcode) -> DecodeStep {
    let element_size = 1u64 << ((raw >> 22) & 0x3);
    let vector_size = if (raw >> 30) != 0 { 16 } else { 8 };
    if element_size >= vector_size {
        return DecodeStep::Reject;
    }
    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: element_size,
        sf: true,
        cond: 0,
        size: vector_size as u8,
    })
}
