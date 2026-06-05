use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBFE0_8400) == 0x2E00_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdExt,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: ((raw >> 11) & 0xF) as u64,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    for (base, op) in [
        (0x0E20_6400, Opcode::SimdSmaxVec),
        (0x0E20_6C00, Opcode::SimdSminVec),
        (0x2E20_6400, Opcode::SimdUmaxVec),
        (0x2E20_6C00, Opcode::SimdUminVec),
        (0x0E20_A400, Opcode::SimdSmaxp),
        (0x0E20_AC00, Opcode::SimdSminp),
        (0x2E20_A400, Opcode::SimdUmaxp),
        (0x2E20_AC00, Opcode::SimdUminp),
    ] {
        if let Some(step) = decode_minmax(raw, base, op) {
            return step;
        }
    }
    if (raw & 0xBF3F_FC00) == 0x0E20_5800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdCnt,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    for (base, op) in [
        (0x0E20_4800, Opcode::SimdCls),
        (0x2E20_4800, Opcode::SimdClz),
    ] {
        if let Some(step) = decode_count_elements(raw, base, op) {
            return step;
        }
    }
    if (raw & 0xBF3F_FC00) == 0x2E20_0800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdRev32,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x0E20_0800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdRev64,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: 0,
                imm: element_size,
                sf: true,
                cond: 0,
                size: if (raw >> 30) != 0 { 16 } else { 8 },
            });
        }
    }
    DecodeStep::Miss
}

fn decode_count_elements(raw: u32, base: u32, op: Opcode) -> Option<DecodeStep> {
    if (raw & 0xBF3F_FC00) != base {
        return None;
    }
    let element_size = 1u64 << ((raw >> 22) & 0x3);
    if element_size == 8 {
        return Some(DecodeStep::Reject);
    }
    Some(DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: element_size,
        sf: true,
        cond: 0,
        size: if (raw >> 30) != 0 { 16 } else { 8 },
    }))
}

fn decode_minmax(raw: u32, base: u32, op: Opcode) -> Option<DecodeStep> {
    if (raw & 0xBF20_FC00) != base {
        return None;
    }
    let element_size = 1u64 << ((raw >> 22) & 0x3);
    if element_size == 8 {
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
        size: if (raw >> 30) != 0 { 16 } else { 8 },
    }))
}
