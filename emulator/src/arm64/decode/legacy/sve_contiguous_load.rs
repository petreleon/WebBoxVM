use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFFC0_E000) == 0x8540_C000 {
        return DecodeStep::Hit(scalar_load(
            raw,
            Opcode::SveLd1rw,
            ((raw >> 16) & 0x3F) * 4,
            4,
        ));
    }
    if (raw & 0xFFF0_E000) == 0xA500_2000 {
        let signed_imm = (((((raw >> 16) & 0xF) as i32) << 28) >> 28) as i64;
        return DecodeStep::Hit(scalar_load(
            raw,
            Opcode::SveLd1rqw,
            signed_imm.wrapping_mul(16) as u32,
            4,
        ));
    }
    if (raw & 0xFF90_E000) == 0xA400_A000 {
        let signed_imm = (((((raw >> 16) & 0xF) as i32) << 28) >> 28) as i64;
        return DecodeStep::Hit(scalar_load(
            raw,
            Opcode::SveLd1b,
            signed_imm as u32,
            1u8 << (((raw >> 21) & 0x3) as u8),
        ));
    }
    DecodeStep::Miss
}

fn scalar_load(raw: u32, op: Opcode, imm: u32, size: u8) -> Instr {
    Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: imm as i32 as i64 as u64,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    }
}
