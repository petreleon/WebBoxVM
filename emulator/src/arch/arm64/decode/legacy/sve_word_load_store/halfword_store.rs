use super::*;

const OFFSET_32: u64 = 2;

pub(super) fn decode(raw: u32) -> Option<Instr> {
    decode_contiguous(raw).or_else(|| decode_scatter(raw))
}

fn decode_contiguous(raw: u32) -> Option<Instr> {
    if let Some(size) = st1h_immediate_size(raw) {
        let signed_imm = (((((raw >> 16) & 0xF) as i32) << 28) >> 28) as i64;
        return Some(Instr {
            op: Opcode::SveSt1h,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: signed_imm as u64,
            sf: false,
            cond: ((raw >> 10) & 0x7) as u8,
            size,
        });
    }
    if (raw & 0xFF80_E000) == 0xE480_4000 {
        let size = 1u8 << (((raw >> 21) & 0x3) as u8);
        if size == 1 {
            return None;
        }
        return Some(Instr {
            op: Opcode::SveSt1h,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: false,
            cond: ((raw >> 10) & 0x7) as u8,
            size,
        });
    }
    None
}

fn decode_scatter(raw: u32) -> Option<Instr> {
    if (raw & 0xFF00_8000) != 0xE400_8000 {
        return None;
    }
    let class = ((raw >> 21) & 0x7, (raw >> 13) & 0x1);
    let (size, scale, offset_32) = match class {
        (0b111, 0) => (4, 1, true),
        (0b110, 0) => (4, 0, true),
        (0b101, 0) => (8, 1, true),
        (0b100, 0) => (8, 0, true),
        (0b101, 1) if ((raw >> 14) & 1) == 0 => (8, 1, false),
        (0b100, 1) if ((raw >> 14) & 1) == 0 => (8, 0, false),
        _ => return None,
    };
    Some(Instr {
        op: Opcode::SveSt1hScatter,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: scale | if offset_32 { OFFSET_32 } else { 0 },
        sf: ((raw >> 14) & 1) != 0,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    })
}

fn st1h_immediate_size(raw: u32) -> Option<u8> {
    match raw & 0xFFF0_E000 {
        0xE4A0_E000 => Some(2),
        0xE4C0_E000 => Some(4),
        0xE4E0_E000 => Some(8),
        _ => None,
    }
}
