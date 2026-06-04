use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let Some((op, src_size, dst_size, elem_size)) = convert_shape(raw) else {
        return DecodeStep::Miss;
    };

    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: dst_size,
        imm: src_size as u64,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size: elem_size,
    })
}

fn convert_shape(raw: u32) -> Option<(Opcode, u8, u8, u8)> {
    Some(match raw & 0xFFFF_E000 {
        0x6594_A000 => (Opcode::SveScvtf, 4, 4, 4),
        0x65D0_A000 => (Opcode::SveScvtf, 4, 8, 8),
        0x65D4_A000 => (Opcode::SveScvtf, 8, 4, 8),
        0x65D6_A000 => (Opcode::SveScvtf, 8, 8, 8),
        0x659C_A000 => (Opcode::SveFcvtzs, 4, 4, 4),
        0x65DC_A000 => (Opcode::SveFcvtzs, 4, 8, 8),
        0x65D8_A000 => (Opcode::SveFcvtzs, 8, 4, 8),
        0x65DE_A000 => (Opcode::SveFcvtzs, 8, 8, 8),
        _ => return None,
    })
}
