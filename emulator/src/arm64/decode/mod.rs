//! AArch64 instruction decoder (pattern-based).

mod system;
mod branch;
mod ldst;
mod data_proc;

use super::opcodes::{Instr, Opcode};

/// Decode a raw 32-bit word into an instruction.
pub fn decode(raw: u32) -> Option<Instr> {
    if raw == 0xD503_201F { return system::decode_nop(); }

    let bits28_24 = (raw >> 24) & 0x1F;
    let bits28_23 = (raw >> 23) & 0x3F;
    let bits28_21 = (raw >> 21) & 0xFF;
    let bits31_26 = (raw >> 26) & 0x3F;
    let bits31_24 = (raw >> 24) & 0xFF;
    let bits31_24_masked_7e = ((raw >> 24) & 0x7E) as u32;

    // Hints / barriers / PAuth: bits[31:12] = 0xD5032
    if ((raw >> 12) & 0xFFFFF) == 0xD5032 {
        let crm = ((raw >> 8) & 0xF) as u8;
        let op2 = ((raw >> 5) & 0x7) as u8;
        if crm == 0b0010 && op2 == 0b011 { return system::decode_wfi(); }
        if crm == 0b0010 && op2 == 0b010 { return system::decode_wfe(); }
        let is_barrier = op2 == 1 && (crm == 0b1010 || crm == 0b1011 || crm == 0b1101);
        if is_barrier { return system::decode_barrier(); }
        return system::decode_nop();
    }

    // MRS/MSR decoding
    let top12 = (raw >> 20) & 0xFFF;
    if top12 == 0xD53 {
        let rd = (raw & 0x1F) as u8;
        let sysreg_id = ((raw >> 5) & 0x7FFF) as u16;
        return Some(Instr { op: Opcode::Mrs, rd, rn: 0, rm: 0, imm: sysreg_id as u64, sf: true, cond: 0, size: 0 });
    }
    if top12 == 0xD51 {
        let rd = (raw & 0x1F) as u8;
        let sysreg_id = ((raw >> 5) & 0x7FFF) as u16;
        return Some(Instr { op: Opcode::Msr, rd, rn: 0, rm: 0, imm: sysreg_id as u64, sf: true, cond: 0, size: 0 });
    }
    if (raw & 0xFFE0001F) == 0xD4000001 {
        let imm16 = ((raw >> 5) & 0xFFFF) as u64;
        return Some(Instr { size: 0, op: Opcode::Svc, rd: 0, rn: 0, rm: 0, imm: imm16, sf: true, cond: 0 });
    }
    if (raw & 0xFFE0001F) == 0xD4200000 {
        let imm16 = ((raw >> 5) & 0xFFFF) as u64;
        return Some(Instr { size: 0, op: Opcode::Brk, rd: 0, rn: 0, rm: 0, imm: imm16, sf: true, cond: 0 });
    }
    if (raw >> 24) == 0xD5 {
        let op0 = (raw >> 19) & 0x3;
        let l = (raw >> 21) & 1;
        let crn = (raw >> 12) & 0xF;
        if l == 0 && op0 == 1 && crn == 8 { return system::decode_tlbi(raw); }
        if (raw & 0xFFFFF000) == 0xD5034000 {
            let daif_bits = (raw & 0xF) as u8;
            return Some(Instr { size: 0, op: Opcode::Nop, rd: 0, rn: 0, rm: 0, imm: daif_bits as u64, sf: true, cond: 1 });
        }
        if ((raw >> 8) & 0xF) == 0b0111 {
            let daif_bits = (raw & 0xF) as u8;
            return Some(Instr { size: 0, op: Opcode::Nop, rd: 0, rn: 0, rm: 0, imm: daif_bits as u64, sf: true, cond: 2 });
        }
        return system::decode_nop();
    }

    if bits28_24 == 0b10000 { return data_proc::decode_adr(raw); }
    if bits28_23 == 0b100010 { return data_proc::decode_addsub_imm(raw); }
    if bits28_23 == 0b100101 {
        let opc = (raw >> 29) & 3;
        if opc == 0 { return data_proc::decode_movn(raw); }
        if opc == 2 { return data_proc::decode_movz(raw); }
        if opc == 3 { return data_proc::decode_movk(raw); }
    }
    if bits28_23 == 0b100100 { return data_proc::decode_logical_imm(raw); }
    if bits28_24 == 0b10011 { return data_proc::decode_bitfield(raw); }
    if bits28_21 == 0b11010100 || bits28_21 == 0b11010010 { return data_proc::decode_condsel(raw); }
    if bits28_21 == 0b11010110 {
        let bit30 = (raw >> 30) & 1;
        if bit30 == 1 { return data_proc::decode_dp_1src(raw); }
        else { return data_proc::decode_dp_2src(raw); }
    }

    let dp_reg_pat = bits28_24;
    if dp_reg_pat == 0b11010 || dp_reg_pat == 0b01011 { return data_proc::decode_dp_register(raw); }

    if bits28_24 == 0b01010 { return data_proc::decode_logical_reg(raw); }

    let ldst_family = (raw >> 24) & 0xF8;
    if ldst_family == 0x38 || ldst_family == 0x78 || ldst_family == 0xB8 || ldst_family == 0xF8 {
        if ((raw >> 22) & 0x3FF) == 0b1111100110 { return system::decode_nop(); }
        return ldst::decode_ldst(raw);
    }

    if ((raw >> 24) & 0xF8) == 0x58 { return ldst::decode_ldr_lit(raw); }

    let ldp_pat = (raw >> 24) & 0x1F;
    if ldp_pat & 0b11100 == 0b01000 && ldp_pat != 0b01011 {
        let is_excl = ((raw >> 29) & 1) == 0;
        if is_excl { return ldst::decode_ldst_excl(raw); }
        else { return ldst::decode_ldst_pair(raw); }
    }

    if bits31_26 == 0b000101 { return branch::decode_b(raw); }
    if bits31_26 == 0b100101 { return branch::decode_bl(raw); }
    if bits31_24 == 0b01010100 { return branch::decode_bcond(raw); }
    if bits31_24_masked_7e == 0b00110100 { return branch::decode_cbz(raw); }
    if bits31_24_masked_7e == 0b00110110 { return branch::decode_tbz(raw); }
    if bits31_24 == 0xD6 { return branch::decode_branch_reg(raw); }
    if bits28_24 == 0b11011 { return data_proc::decode_mul(raw); }

    None
}

#[cfg(test)]
mod tests;
