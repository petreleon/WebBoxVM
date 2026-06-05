use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if matches!(raw & 0xFF3F_FE00, 0x252C_8800 | 0x252D_8800) {
        let rd = (raw & 0x1F) as u8;
        return DecodeStep::Hit(Instr {
            op: if (raw & 0xFF3F_FE00) == 0x252C_8800 {
                Opcode::SveIncpScalar
            } else {
                Opcode::SveDecpScalar
            },
            rd,
            rn: rd,
            rm: 0,
            imm: 0,
            sf: true,
            cond: ((raw >> 5) & 0xF) as u8,
            size: 1 << ((raw >> 22) & 0x3),
        });
    }

    let (op, size) = match raw & 0xFFF0_FC00 {
        0x0430_E000 => (Opcode::SveIncScalar, 1),
        0x0470_E000 => (Opcode::SveIncScalar, 2),
        0x04B0_E000 => (Opcode::SveIncScalar, 4),
        0x04F0_E000 => (Opcode::SveIncScalar, 8),
        0x0430_E400 => (Opcode::SveDecScalar, 1),
        0x0470_E400 => (Opcode::SveDecScalar, 2),
        0x04B0_E400 => (Opcode::SveDecScalar, 4),
        0x04F0_E400 => (Opcode::SveDecScalar, 8),
        _ => return DecodeStep::Miss,
    };

    let rd = (raw & 0x1F) as u8;
    DecodeStep::Hit(Instr {
        op,
        rd,
        rn: rd,
        rm: 0,
        imm: (((raw >> 16) & 0xF) + 1) as u64,
        sf: true,
        cond: ((raw >> 5) & 0x1F) as u8,
        size,
    })
}
