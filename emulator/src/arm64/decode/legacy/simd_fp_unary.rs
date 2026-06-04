use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_simd_fp_binary(raw, 0x2E20_FC00, Opcode::SimdFpDivVec) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x2EA0_D400, Opcode::SimdFpAbd) {
        return DecodeStep::Hit(instr);
    }
    if let Some(step) = decode_simd_fp_abs_compare(raw, 0x2E20_EC00, Opcode::SimdFpFacgeVec) {
        return step;
    }
    if let Some(step) = decode_simd_fp_abs_compare(raw, 0x2EA0_EC00, Opcode::SimdFpFacgtVec) {
        return step;
    }
    if (raw & 0xFF3F_FC00) == 0x7E20_B800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdNeg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 8,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x5EE0_B800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAbs,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 8,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFF3F_FC00) == 0x5E20_B800 {
        return DecodeStep::Reject;
    }
    if (raw & 0xBF3F_FC00) == 0x0E20_B800 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAbs,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x2E20_B800 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdNeg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x0E20_9800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        let q = ((raw >> 30) & 1) != 0;
        if element_size == 8 && !q {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdCmeqZero,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xFF20_FC00) == 0x5E20_9800 {
        if ((raw >> 22) & 0x3) != 0x3 {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdCmeqZero,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 8,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFF3F_FC00) == 0x7E20_8800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size != 8 {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdCmgeZero,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    DecodeStep::Miss
}

fn decode_simd_fp_abs_compare(raw: u32, base: u32, op: Opcode) -> Option<DecodeStep> {
    if (raw & 0xBFA0_FC00) != base {
        return None;
    }
    let q = ((raw >> 30) & 1) != 0;
    let double = ((raw >> 22) & 1) != 0;
    if double && !q {
        return Some(DecodeStep::Reject);
    }
    Some(DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: if double { 8 } else { 4 },
        sf: true,
        cond: 0,
        size: if q { 16 } else { 8 },
    }))
}
