use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBFF8_9C00) == 0x0F00_1400 {
        let shift = (((raw >> 12) & 0xF) >> 1) & 0x3;
        return DecodeStep::Hit(simd_orr_imm(raw, imm8(raw) << (shift * 8), 4));
    }
    if (raw & 0xBFF8_DC00) == 0x0F00_9400 {
        let shift = (((raw >> 12) & 0x2) >> 1) * 8;
        return DecodeStep::Hit(simd_orr_imm(raw, imm8(raw) << shift, 2));
    }
    if (raw & 0xBFF8_EC00) == 0x0F00_C400 {
        let shift = if ((raw >> 12) & 1) == 0 { 8 } else { 16 };
        let ones = (1u64 << shift) - 1;
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: ((imm8(raw) as u64) << shift) | ones,
            sf: true,
            cond: 4,
            size: vector_size(raw),
        });
    }
    DecodeStep::Miss
}

fn simd_orr_imm(raw: u32, imm: u32, element_bytes: u8) -> Instr {
    Instr {
        op: Opcode::SimdOrrImm,
        rd: (raw & 0x1F) as u8,
        rn: 0,
        rm: 0,
        imm: imm as u64,
        sf: true,
        cond: element_bytes,
        size: vector_size(raw),
    }
}

fn imm8(raw: u32) -> u32 {
    ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5)
}

fn vector_size(raw: u32) -> u8 {
    if (raw >> 30) != 0 { 16 } else { 8 }
}
