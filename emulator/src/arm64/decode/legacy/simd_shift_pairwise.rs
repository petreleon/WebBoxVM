use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFF20_FC00) == 0x5E20_8400 {
        if ((raw >> 22) & 0x3) != 0x3 {
            return DecodeStep::Reject;
        }
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAddVec,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: element_size as u8,
        });
    }
    if let Some(instr) = decode_simd_shrn(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_shl(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_sli(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_sri(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_sshr(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_ushr(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_sshll(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_ushll(raw) {
        return DecodeStep::Hit(instr);
    }
    if (raw & 0xFF3F_FC00) == 0x0E21_2800 {
        let dest_element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdXtn,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: dest_element_size,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x5EF1_B800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAddp,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: 8,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if let Some(step) = decode_simd_narrow_high(raw, 0x0E20_4000, Opcode::SimdAddhn) {
        return step;
    }
    if let Some(step) = decode_simd_narrow_high(raw, 0x0E20_6000, Opcode::SimdSubhn) {
        return step;
    }
    if (raw & 0xBF20_FC00) == 0x0E20_BC00 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAddp,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x0E31_B800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAddv,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x2E30_A800 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 || (element_size == 4 && !q) {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdUmaxv,
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

fn decode_simd_narrow_high(raw: u32, base: u32, op: Opcode) -> Option<DecodeStep> {
    if (raw & 0xFF20_FC00) != base {
        return None;
    }
    let size = (raw >> 22) & 0x3;
    if size == 0x3 {
        return Some(DecodeStep::Reject);
    }
    let dest_element_size = 1u64 << size;
    Some(DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: dest_element_size,
        sf: false,
        cond: 0,
        size: 8,
    }))
}
