use super::helpers::{bitfield_mask, bitfield_size};
use super::opcodes::*;
use super::*;

impl WasmExpr {
    pub(super) fn emit_bitfield_extract(&mut self, instr: crate::arm64::Instr, signed: bool) {
        let size = bitfield_size(instr.sf);
        let r = instr.rm as u32;
        let s = instr.imm as u32;
        self.emit_read_reg(instr.rn, instr.sf);
        if s >= r {
            self.emit_right_shift(r);
            self.emit_and_mask(bitfield_mask(s - r + 1));
            self.emit_sign_extend_if_needed(signed, s - r);
        } else {
            let shift = size - r;
            self.emit_and_mask(bitfield_mask(s + 1));
            self.emit_left_shift(shift);
            self.emit_sign_extend_if_needed(signed, shift + s);
        }
    }

    pub(super) fn emit_bitfield_insert(&mut self, instr: crate::arm64::Instr) {
        let size = bitfield_size(instr.sf);
        let r = instr.rm as u32;
        let s = instr.imm as u32;
        if s >= r {
            let mask = bitfield_mask(s - r + 1);
            self.emit_read_reg(instr.rd, instr.sf);
            self.emit_and_mask(!mask);
            self.emit_read_reg(instr.rn, instr.sf);
            self.emit_right_shift(r);
            self.emit_and_mask(mask);
            self.op(OP_I64_OR);
        } else {
            let shift = size - r;
            let mask = bitfield_mask(s + 1);
            self.emit_read_reg(instr.rd, instr.sf);
            self.emit_and_mask(!(mask << shift));
            self.emit_read_reg(instr.rn, instr.sf);
            self.emit_and_mask(mask);
            self.emit_left_shift(shift);
            self.op(OP_I64_OR);
        }
    }

    fn emit_and_mask(&mut self, mask: u64) {
        self.i64_const(mask);
        self.op(OP_I64_AND);
    }

    fn emit_left_shift(&mut self, amount: u32) {
        if amount == 0 {
            return;
        }
        self.i64_const(amount as u64);
        self.op(OP_I64_SHL);
    }

    fn emit_right_shift(&mut self, amount: u32) {
        if amount == 0 {
            return;
        }
        self.i64_const(amount as u64);
        self.op(OP_I64_SHR_U);
    }

    fn emit_sign_extend_if_needed(&mut self, signed: bool, sign_bit: u32) {
        if !signed {
            return;
        }
        let shift = 63 - sign_bit;
        self.emit_left_shift(shift);
        self.i64_const(shift as u64);
        self.op(OP_I64_SHR_S);
    }
}
