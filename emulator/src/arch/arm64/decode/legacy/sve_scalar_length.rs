use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let add_len_base = raw & 0xFFE0_F800;
    if matches!(
        add_len_base,
        0x0420_5000 | 0x0420_5800 | 0x0460_5000 | 0x0460_5800
    ) {
        return scalar_len_instr(
            raw,
            match add_len_base {
                0x0420_5000 => Opcode::SveAddvl,
                0x0420_5800 => Opcode::SveAddsvl,
                0x0460_5000 => Opcode::SveAddpl,
                _ => Opcode::SveAddspl,
            },
            ((raw >> 16) & 0x1F) as u8,
        );
    }

    let rd_len_base = raw & 0xFFFF_F800;
    if matches!(rd_len_base, 0x04BF_5000 | 0x04BF_5800) {
        return scalar_len_instr(
            raw,
            if rd_len_base == 0x04BF_5000 {
                Opcode::SveRdvl
            } else {
                Opcode::SveRdsvl
            },
            0,
        );
    }
    DecodeStep::Miss
}

fn scalar_len_instr(raw: u32, op: Opcode, rn: u8) -> DecodeStep {
    let imm6 = ((raw >> 5) & 0x3F) as u8;
    let signed_imm = ((imm6 as i8) << 2) >> 2;
    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn,
        rm: 0,
        imm: signed_imm as i64 as u64,
        sf: true,
        cond: 0,
        size: 0,
    })
}
