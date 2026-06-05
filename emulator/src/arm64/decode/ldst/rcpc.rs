use super::*;

pub(in crate::arm64::decode) fn decode_ldapur(raw: u32) -> Option<Instr> {
    let (op, size, sf) = match raw & 0xFFE0_0C00 {
        0x1940_0000 => (Opcode::Ldapur, 1, false),
        0x5940_0000 => (Opcode::Ldapur, 2, false),
        0x9940_0000 => (Opcode::Ldapur, 4, false),
        0xD940_0000 => (Opcode::Ldapur, 8, true),
        0x1980_0000 => (Opcode::Ldapurs, 1, true),
        0x19C0_0000 => (Opcode::Ldapurs, 1, false),
        0x5980_0000 => (Opcode::Ldapurs, 2, true),
        0x59C0_0000 => (Opcode::Ldapurs, 2, false),
        0x9980_0000 => (Opcode::Ldapurs, 4, true),
        _ => return None,
    };

    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: simm9(raw) as u64,
        sf,
        cond: 0,
        size,
    })
}

pub(in crate::arm64::decode) fn decode_stlur(raw: u32) -> Option<Instr> {
    let (size, sf) = match raw & 0xFFE0_0C00 {
        0x1900_0000 => (1, false),
        0x5900_0000 => (2, false),
        0x9900_0000 => (4, false),
        0xD900_0000 => (8, true),
        _ => return None,
    };

    Some(Instr {
        op: Opcode::Stlur,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: simm9(raw) as u64,
        sf,
        cond: 0,
        size,
    })
}

fn simm9(raw: u32) -> i64 {
    let imm = ((raw >> 12) & 0x1FF) as i64;
    if imm & 0x100 != 0 {
        imm - 0x200
    } else {
        imm
    }
}
