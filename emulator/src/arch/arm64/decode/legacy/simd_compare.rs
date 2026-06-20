use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    for (base, op) in [
        (0x0E20_3400, Opcode::SimdCmgtReg),
        (0x0E20_3C00, Opcode::SimdCmgeReg),
        (0x2E20_8C00, Opcode::SimdCmeqReg),
        (0x2E20_3400, Opcode::SimdCmhiReg),
        (0x2E20_3C00, Opcode::SimdCmhsReg),
    ] {
        if let Some(step) = decode_simd_compare_reg(raw, base, op) {
            return step;
        }
    }
    for (base, op) in [
        (0x5EE0_3400, Opcode::SimdCmgtReg),
        (0x5EE0_3C00, Opcode::SimdCmgeReg),
        (0x5EE0_8C00, Opcode::SimdCmtst),
        (0x7EE0_3400, Opcode::SimdCmhiReg),
        (0x7EE0_3C00, Opcode::SimdCmhsReg),
        (0x7EE0_8C00, Opcode::SimdCmeqReg),
    ] {
        if let Some(instr) = decode_scalar_compare_reg(raw, base, op) {
            return DecodeStep::Hit(instr);
        }
    }
    if (raw & 0xFF20_FC00) == 0x7E20_2C00 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdUqsub,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: element_size as u8,
        });
    }
    if let Some(step) = decode_simd_compare_reg(raw, 0x2E20_0C00, Opcode::SimdUqadd) {
        return step;
    }
    if (raw & 0xBF20_FC00) == 0x2E20_3800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdShll,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: 0,
                imm: element_size * 8,
                sf: (raw >> 30) != 0,
                cond: element_size as u8,
                size: 16,
            });
        }
    }
    if (raw & 0xBF20_FC00) == 0x0E20_0000 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdSaddl,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: ((raw >> 16) & 0x1F) as u8,
                imm: 0,
                sf: (raw >> 30) != 0,
                cond: element_size as u8,
                size: 16,
            });
        }
    }
    if (raw & 0xBF20_FC00) == 0x2E20_0000 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdUaddl,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: ((raw >> 16) & 0x1F) as u8,
                imm: 0,
                sf: (raw >> 30) != 0,
                cond: element_size as u8,
                size: 16,
            });
        }
    }
    if (raw & 0xBF20_FC00) == 0x2E20_2000 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdUsubl,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: ((raw >> 16) & 0x1F) as u8,
                imm: 0,
                sf: (raw >> 30) != 0,
                cond: element_size as u8,
                size: 16,
            });
        }
    }
    DecodeStep::Miss
}

fn decode_scalar_compare_reg(raw: u32, base: u32, op: Opcode) -> Option<Instr> {
    if (raw & 0xFFE0_FC00) != base {
        return None;
    }
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 8,
        sf: true,
        cond: 0,
        size: 8,
    })
}

fn decode_simd_compare_reg(raw: u32, base: u32, op: Opcode) -> Option<DecodeStep> {
    if (raw & 0xBF20_FC00) != base {
        return None;
    }
    let q = (raw >> 30) != 0;
    let element_size = 1u64 << ((raw >> 22) & 0x3);
    if element_size == 8 && !q {
        return Some(DecodeStep::Reject);
    }
    Some(DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: element_size,
        sf: true,
        cond: 0,
        size: if q { 16 } else { 8 },
    }))
}
