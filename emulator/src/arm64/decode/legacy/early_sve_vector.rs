use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if raw == 0xD503_201F {
        return DecodeStep::from_option(system::decode_nop());
    }
    if let Some(instr) = decode_simd_ld1_multi(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_ldst1_lane(raw) {
        return DecodeStep::Hit(instr);
    }
    if (raw & 0xFFC0_E000) == 0x85C0_E000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SveLd1rd,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: (((raw >> 16) & 0x3F) * 8) as u64,
            sf: true,
            cond: ((raw >> 10) & 0x7) as u8,
            size: 8,
        });
    }
    if (raw & 0xFFF0_E000) == 0xA580_2000 {
        let signed_imm = (((((raw >> 16) & 0xF) as i32) << 28) >> 28) as i64;
        return DecodeStep::Hit(Instr {
            op: Opcode::SveLd1rqd,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: signed_imm.wrapping_mul(16) as u64,
            sf: true,
            cond: ((raw >> 10) & 0x7) as u8,
            size: 8,
        });
    }
    if (raw & 0xFFE0_E000) == 0xC5E0_C000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SveLd1d,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: ((raw >> 10) & 0x7) as u8,
            size: 8,
        });
    }
    if (raw & 0xFFF0_E000) == 0xA5E0_A000 || (raw & 0xFFF0_E000) == 0xE5E0_E000 {
        let signed_imm = (((((raw >> 16) & 0xF) as i32) << 28) >> 28) as i64;
        return DecodeStep::Hit(Instr {
            op: if (raw & 0x4000_0000) == 0 {
                Opcode::SveLd1d
            } else {
                Opcode::SveSt1d
            },
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: signed_imm as u64,
            sf: true,
            cond: ((raw >> 10) & 0x7) as u8,
            size: 8,
        });
    }
    let sve_ldst_base = raw & 0xFFC0_E000;
    if matches!(
        sve_ldst_base,
        0x8580_0000 | 0x8580_4000 | 0xE580_0000 | 0xE580_4000
    ) {
        let imm9 = ((((raw >> 16) & 0x3F) << 3) | ((raw >> 10) & 0x7)) as u16;
        let signed_imm = ((imm9 as i16) << 7) >> 7;
        return DecodeStep::Hit(Instr {
            op: if (raw & 0x4000_0000) == 0 {
                Opcode::SveLdr
            } else {
                Opcode::SveStr
            },
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: signed_imm as i64 as u64,
            sf: true,
            cond: if (raw & 0x4000) != 0 { 1 } else { 0 },
            size: 0,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x0420_BC00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SveMovprfx,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: false,
            cond: 0xFF,
            size: 0,
        });
    }
    if (raw & 0xFF3E_E000) == 0x0410_2000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SveMovprfx,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: (raw & 0x0001_0000) != 0,
            cond: ((raw >> 10) & 0x7) as u8,
            size: 1u8 << (((raw >> 22) & 0x3) as u8),
        });
    }
    if (raw & 0xFF3F_FC00) == 0x0520_3800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SveDupGpr,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 1u8 << (((raw >> 22) & 0x3) as u8),
        });
    }
    let sve_addsub_base = raw & 0xFF20_FC00;
    if sve_addsub_base == 0x0420_0000 || sve_addsub_base == 0x0420_0400 {
        return DecodeStep::Hit(Instr {
            op: if sve_addsub_base == 0x0420_0000 {
                Opcode::SveAddVec
            } else {
                Opcode::SveSubVec
            },
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: false,
            cond: 0,
            size: 1u8 << (((raw >> 22) & 0x3) as u8),
        });
    }
    DecodeStep::Miss
}
