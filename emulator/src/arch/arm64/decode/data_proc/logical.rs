use super::*;

pub(in crate::arch::arm64::decode) fn decode_logical_reg(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let opc = (raw >> 29) & 3;
    let shift = ((raw >> 22) & 3) as u8;
    let n = ((raw >> 21) & 1) != 0;
    let rm = ((raw >> 16) & 0x1F) as u8;
    let imm6 = ((raw >> 10) & 0x3F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    if rn == 31 && opc == 1 && shift == 0 && !n && imm6 == 0 {
        return Some(Instr {
            size: 0,
            op: Opcode::MovReg,
            rd,
            rn: 0,
            rm,
            imm: 0,
            sf,
            cond: 0,
        });
    }

    let op = match opc {
        0 => Opcode::AndReg,
        1 => Opcode::OrrReg,
        2 => Opcode::EorReg,
        3 => Opcode::AndsReg,
        _ => return None,
    };

    let cond = ((n as u8) << 2) | shift;
    Some(Instr {
        size: 0,
        op,
        rd,
        rn,
        rm,
        imm: imm6 as u64,
        sf,
        cond,
    })
}

pub(in crate::arch::arm64::decode) fn decode_logical_imm(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let opc = (raw >> 29) & 0x3;
    let n = (raw >> 22) & 1;
    let immr = ((raw >> 16) & 0x3F) as u32;
    let imms = ((raw >> 10) & 0x3F) as u32;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    let imm = decode_bitmask_imm(n, immr, imms, sf)?;
    let op = match opc {
        0 => Opcode::AndImm,
        1 => Opcode::OrrImm,
        2 => Opcode::EorImm,
        3 => Opcode::AndsImm,
        _ => return None,
    };
    Some(Instr {
        size: 0,
        op,
        rd,
        rn,
        rm: 0,
        imm,
        sf,
        cond: 0,
    })
}
