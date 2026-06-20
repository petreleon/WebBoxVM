use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_simd_fp_binary(raw, 0x2E20_FC00, Opcode::SimdFpDivVec) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x2EA0_D400, Opcode::SimdFpAbd) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_fp_long_narrow(
        raw,
        0x0E21_7800,
        Opcode::SimdFcvtl,
        Opcode::SimdFcvtl2,
        true,
    ) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_fp_long_narrow(
        raw,
        0x0E21_6800,
        Opcode::SimdFcvtn,
        Opcode::SimdFcvtn2,
        false,
    ) {
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
    DecodeStep::Miss
}

fn decode_simd_fp_long_narrow(
    raw: u32,
    base: u32,
    low_op: Opcode,
    high_op: Opcode,
    long: bool,
) -> Option<Instr> {
    if (raw & 0xBFBF_FC00) != base {
        return None;
    }
    let high = ((raw >> 30) & 1) != 0;
    let narrow_size = if ((raw >> 22) & 1) != 0 { 4 } else { 2 };
    let (src_size, dst_size, vector_size) = if long {
        (narrow_size, narrow_size * 2, 16)
    } else {
        (narrow_size * 2, narrow_size, 8)
    };
    Some(Instr {
        op: if high { high_op } else { low_op },
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: src_size,
        sf: high,
        cond: dst_size as u8,
        size: vector_size,
    })
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
