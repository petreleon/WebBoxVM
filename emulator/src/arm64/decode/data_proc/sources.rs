use super::*;

pub(in crate::arm64::decode) fn decode_dp_1src(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let opcode2 = ((raw >> 16) & 0x1F) as u8;
    let opcode = ((raw >> 10) & 0x3F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    if opcode2 == 0 {
        match opcode {
            0b000000 => Some(Instr {
                size: 0,
                op: Opcode::Rbit,
                rd,
                rn,
                rm: 0,
                imm: 0,
                sf,
                cond: 0,
            }),
            0b000001 => Some(Instr {
                size: 0,
                op: Opcode::Rev16,
                rd,
                rn,
                rm: 0,
                imm: 0,
                sf,
                cond: 0,
            }),
            0b000010 => {
                let op = if sf { Opcode::Rev32 } else { Opcode::Rev };
                Some(Instr {
                    size: 0,
                    op,
                    rd,
                    rn,
                    rm: 0,
                    imm: 0,
                    sf,
                    cond: 0,
                })
            }
            0b000011 => Some(Instr {
                size: 0,
                op: Opcode::Rev,
                rd,
                rn,
                rm: 0,
                imm: 0,
                sf,
                cond: 0,
            }),
            0b000100 => Some(Instr {
                size: 0,
                op: Opcode::Clz,
                rd,
                rn,
                rm: 0,
                imm: 0,
                sf,
                cond: 0,
            }),
            _ => None,
        }
    } else {
        None
    }
}

pub(in crate::arm64::decode) fn decode_dp_2src(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let rm = ((raw >> 16) & 0x1F) as u8;
    let opcode = ((raw >> 10) & 0x3F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    if (0b010000..=0b010011).contains(&opcode) {
        let size = 1u8 << (opcode - 0b010000);
        if sf != (size == 8) {
            return None;
        }
        return Some(Instr {
            size,
            op: Opcode::Crc32,
            rd,
            rn,
            rm,
            imm: 0,
            sf: false,
            cond: 0,
        });
    }

    match opcode {
        0b000010 => Some(Instr {
            size: 0,
            op: Opcode::Udiv,
            rd,
            rn,
            rm,
            imm: 0,
            sf,
            cond: 0,
        }),
        0b000011 => Some(Instr {
            size: 0,
            op: Opcode::Sdiv,
            rd,
            rn,
            rm,
            imm: 0,
            sf,
            cond: 0,
        }),
        0b001000 => Some(Instr {
            size: 0,
            op: Opcode::Lslv,
            rd,
            rn,
            rm,
            imm: 0,
            sf,
            cond: 0,
        }),
        0b001001 => Some(Instr {
            size: 0,
            op: Opcode::Lsrv,
            rd,
            rn,
            rm,
            imm: 0,
            sf,
            cond: 0,
        }),
        0b001010 => Some(Instr {
            size: 0,
            op: Opcode::Asrv,
            rd,
            rn,
            rm,
            imm: 0,
            sf,
            cond: 0,
        }),
        0b001011 => Some(Instr {
            size: 0,
            op: Opcode::Rorv,
            rd,
            rn,
            rm,
            imm: 0,
            sf,
            cond: 0,
        }),
        _ => None,
    }
}
