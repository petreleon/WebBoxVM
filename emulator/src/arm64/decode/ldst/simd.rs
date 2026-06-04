use super::*;

pub(in crate::arm64::decode) fn decode_simd_ldst(raw: u32) -> Option<Instr> {
    let size_field = (raw >> 30) & 3;
    let opc = (raw >> 22) & 3;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let (bytes, is_load) = match (size_field, opc) {
        (0, 0) => (1u8, false),
        (0, 1) => (1u8, true),
        (0, 2) => (16u8, false),
        (0, 3) => (16u8, true),
        (1, 0) => (2u8, false),
        (1, 1) => (2u8, true),
        (2, 0) => (4u8, false),
        (2, 1) => (4u8, true),
        (3, 0) => (8u8, false),
        (3, 1) => (8u8, true),
        _ => return None,
    };
    let op = if is_load {
        Opcode::SimdLdr
    } else {
        Opcode::SimdStr
    };

    let bit24 = (raw >> 24) & 1;
    if bit24 == 1 {
        let imm12 = ((raw >> 10) & 0xFFF) as u64;
        return Some(Instr {
            size: bytes,
            op,
            rd: rt,
            rn,
            rm: 0xFF,
            imm: imm12 << bytes.trailing_zeros(),
            sf: bytes >= 8,
            cond: 0,
        });
    }

    let bit21 = (raw >> 21) & 1;
    let bits11_10 = (raw >> 10) & 3;
    let raw9 = (raw >> 12) & 0x1FF;
    let simm9 = if raw9 & 0x100 != 0 {
        (raw9 as i64) - 0x200
    } else {
        raw9 as i64
    };

    if bit21 == 0 && bits11_10 == 0b00 {
        return Some(Instr {
            size: bytes,
            op,
            rd: rt,
            rn,
            rm: 0xFF,
            imm: simm9 as u64,
            sf: bytes >= 8,
            cond: 0,
        });
    }
    if bit21 == 0 && bits11_10 == 0b01 {
        return Some(Instr {
            size: bytes,
            op,
            rd: rt,
            rn,
            rm: 0xFF,
            imm: simm9 as u64,
            sf: bytes >= 8,
            cond: 1,
        });
    }
    if bit21 == 0 && bits11_10 == 0b11 {
        return Some(Instr {
            size: bytes,
            op,
            rd: rt,
            rn,
            rm: 0xFF,
            imm: simm9 as u64,
            sf: bytes >= 8,
            cond: 3,
        });
    }
    if bit21 == 1 && bits11_10 == 0b10 {
        let rm = ((raw >> 16) & 0x1F) as u8;
        let option = ((raw >> 13) & 7) as u8;
        let s = ((raw >> 12) & 1) as u64;
        return Some(Instr {
            size: bytes,
            op,
            rd: rt,
            rn,
            rm,
            imm: s,
            sf: bytes >= 8,
            cond: option,
        });
    }

    None
}
