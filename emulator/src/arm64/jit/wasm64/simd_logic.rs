use super::opcodes::*;
use super::*;
use crate::arm64::{Instr, Opcode};

impl WasmExpr {
    pub(super) fn emit_simd_logic_reg(&mut self, instr: Instr) -> bool {
        if instr.size != 8 && instr.size != 16 {
            return false;
        }
        if !is_logic_reg(instr.op) {
            return false;
        }

        self.emit_simd_logic_half(instr, false);
        if instr.size == 16 {
            self.emit_simd_logic_half(instr, true);
        } else {
            self.emit_write_simd_half_with(instr.rd, true, |this| this.i64_const(0));
        }
        true
    }

    fn emit_simd_logic_half(&mut self, instr: Instr, high: bool) {
        self.emit_write_simd_half_with(instr.rd, high, |this| {
            this.emit_logic_half_value(instr, high);
        });
    }

    fn emit_logic_half_value(&mut self, instr: Instr, high: bool) {
        match instr.op {
            Opcode::SimdAnd => {
                self.emit_binary_logic(instr.rn, instr.rm, high, OP_I64_AND, false)
            }
            Opcode::SimdBic => {
                self.emit_binary_logic(instr.rn, instr.rm, high, OP_I64_AND, true)
            }
            Opcode::SimdOrr => self.emit_binary_logic(instr.rn, instr.rm, high, OP_I64_OR, false),
            Opcode::SimdOrn => self.emit_binary_logic(instr.rn, instr.rm, high, OP_I64_OR, true),
            Opcode::SimdEor => {
                self.emit_binary_logic(instr.rn, instr.rm, high, OP_I64_XOR, false)
            }
            Opcode::SimdBsl => self.emit_mask_logic(instr.rd, instr.rn, instr.rm, high),
            Opcode::SimdBit => self.emit_mask_logic(instr.rm, instr.rn, instr.rd, high),
            Opcode::SimdBif => self.emit_mask_logic(instr.rm, instr.rd, instr.rn, high),
            _ => unreachable!(),
        }
    }

    fn emit_binary_logic(&mut self, rn: u8, rm: u8, high: bool, op: u8, invert_rm: bool) {
        self.emit_read_simd_half(rn, high);
        self.emit_read_simd_half(rm, high);
        if invert_rm {
            self.emit_not();
        }
        self.op(op);
    }

    fn emit_mask_logic(&mut self, mask: u8, src_true: u8, src_false: u8, high: bool) {
        self.emit_read_simd_half(src_true, high);
        self.emit_read_simd_half(mask, high);
        self.op(OP_I64_AND);
        self.emit_read_simd_half(src_false, high);
        self.emit_read_simd_half(mask, high);
        self.emit_not();
        self.op(OP_I64_AND);
        self.op(OP_I64_OR);
    }

    fn emit_not(&mut self) {
        self.i64_const(u64::MAX);
        self.op(OP_I64_XOR);
    }
}

pub(super) fn is_logic_reg(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SimdAnd
            | Opcode::SimdBic
            | Opcode::SimdOrr
            | Opcode::SimdOrn
            | Opcode::SimdEor
            | Opcode::SimdBsl
            | Opcode::SimdBit
            | Opcode::SimdBif
    )
}
