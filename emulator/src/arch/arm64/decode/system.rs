//! System instruction decoders: NOP, WFI/WFE, TLBI, barriers.

use super::{Instr, Opcode};

pub(super) fn decode_nop() -> Option<Instr> {
    Some(simple(Opcode::Nop, 0))
}

pub(super) fn decode_yield() -> Option<Instr> {
    Some(simple(Opcode::Yield, 4))
}

pub(super) fn decode_wfi() -> Option<Instr> {
    Some(simple(Opcode::Wfi, 0))
}

pub(super) fn decode_wfe() -> Option<Instr> {
    Some(simple(Opcode::Wfe, 0))
}

pub(super) fn decode_wait_timeout(raw: u32, op: Opcode) -> Option<Instr> {
    Some(Instr {
        rd: (raw & 0x1F) as u8,
        ..simple(op, 0)
    })
}

pub(super) fn decode_clrex() -> Option<Instr> {
    Some(simple(Opcode::Clrex, 3))
}

pub(super) fn decode_cfinv() -> Option<Instr> {
    Some(simple(Opcode::Cfinv, 0))
}

pub(super) fn decode_rmif(raw: u32) -> Option<Instr> {
    Some(Instr {
        op: Opcode::Rmif,
        rd: 0,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: ((raw >> 15) & 0x3F) as u64,
        sf: true,
        cond: (raw & 0xF) as u8,
        size: 0,
    })
}

pub(super) fn decode_setf(raw: u32, op: Opcode) -> Option<Instr> {
    Some(Instr {
        op,
        rd: 0,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: 0,
        sf: false,
        cond: 0,
        size: 0,
    })
}

pub(super) fn decode_tlbi(raw: u32) -> Option<Instr> {
    let op1 = ((raw >> 16) & 0x7) as u8;
    let crm = ((raw >> 8) & 0xF) as u8;
    let op2 = ((raw >> 5) & 0x7) as u8;
    let rt = (raw & 0x1F) as u8;
    let variant = ((op1 as u64) << 16) | ((crm as u64) << 8) | ((op2 as u64) << 4) | (rt as u64);
    Some(Instr {
        op: Opcode::Tlbi,
        rd: 0,
        rn: 0,
        rm: 0,
        imm: variant,
        sf: true,
        cond: 0,
        size: 0,
    })
}

pub(super) fn decode_dc_zva(raw: u32) -> Option<Instr> {
    Some(Instr {
        op: Opcode::DcZva,
        rd: (raw & 0x1F) as u8,
        rn: 0,
        rm: 0,
        imm: 0,
        sf: true,
        cond: 0,
        size: 0,
    })
}

pub(super) fn decode_barrier(op: Opcode) -> Option<Instr> {
    Some(simple(op, 0))
}

pub(super) fn decode_cache_maintenance(raw: u32) -> Option<Instr> {
    Some(Instr {
        op: Opcode::NopBarrier,
        rd: (raw & 0x1F) as u8,
        rn: 0,
        rm: 0,
        imm: (((raw >> 5) & 0x7FFF) as u64),
        sf: true,
        cond: 0,
        size: 0,
    })
}

pub(super) fn decode_udf(raw: u32) -> Option<Instr> {
    Some(Instr {
        op: Opcode::Udf,
        rd: 0,
        rn: 0,
        rm: 0,
        imm: (raw & 0xFFFF) as u64,
        sf: true,
        cond: 0,
        size: 0,
    })
}

pub(super) fn decode_extension_nop(op: Opcode, rd: u8) -> Option<Instr> {
    Some(Instr {
        op,
        rd,
        rn: 0,
        rm: 0,
        imm: 0,
        sf: true,
        cond: 0,
        size: 0,
    })
}

fn simple(op: Opcode, cond: u8) -> Instr {
    Instr {
        op,
        rd: 0,
        rn: 0,
        rm: 0,
        imm: 0,
        sf: true,
        cond,
        size: 0,
    }
}
