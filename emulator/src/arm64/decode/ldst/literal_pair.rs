use super::*;

pub(in crate::arm64::decode) fn decode_ldr_lit(raw: u32) -> Option<Instr> {
    let opc = (raw >> 30) & 0x3;
    let simd = ((raw >> 26) & 1) != 0;
    let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
    let offset = (imm19 << 13) >> 11;
    let rt = (raw & 0x1F) as u8;
    if opc == 0b11 {
        if simd {
            return None;
        }
        return Some(Instr {
            size: 0,
            op: Opcode::Nop,
            rd: 0,
            rn: 0,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
        });
    }

    let size = if simd { 4u8 << opc } else { 0 };
    Some(Instr {
        size,
        op: Opcode::LdrLit,
        rd: rt,
        rn: 0,
        rm: 0,
        imm: offset as u64,
        sf: opc != 0 || simd,
        cond: if !simd && opc == 0b10 { 1 } else { 0 },
    })
}

pub(in crate::arm64::decode) fn decode_ldst_pair(raw: u32) -> Option<Instr> {
    let opc = (raw >> 30) & 0b11;
    let l = ((raw >> 22) & 1) != 0;
    let op2 = ((raw >> 23) & 0x3) as u8;
    let imm7_raw = (raw >> 15) & 0x7F;
    let imm7 = if imm7_raw & 0x40 != 0 {
        (imm7_raw as i64) - 0x80
    } else {
        imm7_raw as i64
    };
    let rt2 = ((raw >> 10) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let v = ((raw >> 26) & 1) != 0;
    let bytes = if v {
        match opc {
            0 => 4,
            1 => 8,
            2 => 16,
            _ => return None,
        }
    } else {
        match opc {
            0 => 4,
            1 if l => 4,
            2 => 8,
            _ => return None,
        }
    };
    let offset = imm7 * bytes as i64;
    let op = if v {
        if l { Opcode::SimdLdp } else { Opcode::SimdStp }
    } else {
        if l && opc == 1 {
            Opcode::Ldpsw
        } else if l {
            Opcode::Ldp
        } else {
            Opcode::Stp
        }
    };
    Some(Instr {
        size: if v { bytes } else { 0 },
        op,
        rd: rt,
        rn,
        rm: rt2,
        imm: offset as u64,
        sf: bytes == 8,
        cond: op2,
    })
}
