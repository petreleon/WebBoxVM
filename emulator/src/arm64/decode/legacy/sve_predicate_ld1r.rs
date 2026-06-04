use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let sve_logical_base = raw & 0xFFE0_FC00;
    if sve_logical_base == 0x0460_3000 || sve_logical_base == 0x04A0_3000 {
        return DecodeStep::Hit(Instr {
            op: if sve_logical_base == 0x0460_3000 {
                Opcode::SveOrrVec
            } else {
                Opcode::SveEorVec
            },
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: false,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFF20_C000) == 0x0520_C000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SveSel,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: false,
            cond: ((raw >> 10) & 0xF) as u8,
            size: 1u8 << (((raw >> 22) & 0x3) as u8),
        });
    }
    if (raw & 0xFF3F_FC10) == 0x2518_E000 {
        let size_bits = ((raw >> 22) & 0x3) as u8;
        return DecodeStep::Hit(Instr {
            op: Opcode::SvePtrue,
            rd: (raw & 0xF) as u8,
            rn: 0,
            rm: 0,
            imm: 0,
            sf: false,
            cond: ((raw >> 5) & 0x1F) as u8,
            size: 1u8 << size_bits,
        });
    }
    if (raw & 0xFFFF_C21F) == 0x2550_C000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SvePtest,
            rd: ((raw >> 10) & 0xF) as u8,
            rn: ((raw >> 5) & 0xF) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 1,
        });
    }
    let pred_logical_base = raw & 0xFFF0_C210;
    if matches!(
        pred_logical_base,
        0x2500_4000 | 0x2540_4000 | 0x2580_4000 | 0x25C0_4000
    ) {
        return DecodeStep::Hit(Instr {
            op: if (raw & 0x0080_0000) == 0 {
                Opcode::SvePredAnd
            } else {
                Opcode::SvePredOrr
            },
            rd: (raw & 0xF) as u8,
            rn: ((raw >> 5) & 0xF) as u8,
            rm: ((raw >> 16) & 0xF) as u8,
            imm: 0,
            sf: (raw & 0x0040_0000) != 0,
            cond: ((raw >> 10) & 0xF) as u8,
            size: 1,
        });
    }
    if (raw & 0xFF20_E000) == 0x0420_E000 {
        let size_bits = ((raw >> 22) & 0x3) as u8;
        return DecodeStep::Hit(Instr {
            op: Opcode::SveCnt,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: (((raw >> 16) & 0xF) + 1) as u64,
            sf: true,
            cond: ((raw >> 5) & 0x1F) as u8,
            size: 1u8 << size_bits,
        });
    }
    if (raw & 0xFFE0_F800) == 0x0420_5000 || (raw & 0xFFE0_F800) == 0x0420_5800 {
        let imm6 = ((raw >> 5) & 0x3F) as u8;
        let signed_imm = ((imm6 as i8) << 2) >> 2;
        return DecodeStep::Hit(Instr {
            op: if (raw & 0x800) == 0 {
                Opcode::SveAddvl
            } else {
                Opcode::SveAddsvl
            },
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 16) & 0x1F) as u8,
            rm: 0,
            imm: signed_imm as i64 as u64,
            sf: true,
            cond: 0,
            size: 0,
        });
    }
    let ld1r_no_offset = (raw & 0xBFFF_F000) == 0x0D40_C000;
    let ld1r_post_index = (raw & 0xBFE0_F000) == 0x0DC0_C000;
    if ld1r_no_offset || ld1r_post_index {
        let element_size = 1u8 << (((raw >> 10) & 0x3) as u8);
        let rm_field = ((raw >> 16) & 0x1F) as u8;
        let (rm, imm) = if ld1r_post_index {
            if rm_field == 31 {
                (0xFE, element_size as u64)
            } else {
                (rm_field, 0)
            }
        } else {
            (0xFF, 0)
        };
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdLd1r,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm,
            imm,
            sf: true,
            cond: element_size,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    DecodeStep::Miss
}
