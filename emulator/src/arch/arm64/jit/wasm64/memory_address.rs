use super::opcodes::*;
use super::*;
use crate::arch::arm64::Instr;

pub(super) const ADDR_LOCAL: u32 = 1;
pub(super) const WRITEBACK_LOCAL: u32 = 2;
pub(super) const VALUE_LOCAL: u32 = 3;

impl WasmExpr {
    pub(super) fn emit_memory_address(&mut self, instr: Instr) -> Option<bool> {
        if instr.rm == 0xFF {
            return Some(self.emit_immediate_address(instr));
        }
        self.emit_read_base(instr.rn, true);
        self.emit_register_offset(instr)?;
        self.op(OP_I64_ADD);
        self.local_set(ADDR_LOCAL);
        Some(false)
    }

    fn emit_immediate_address(&mut self, instr: Instr) -> bool {
        self.emit_read_base(instr.rn, true);
        match instr.cond {
            1 => {
                self.local_set(ADDR_LOCAL);
                self.local_get(ADDR_LOCAL);
                self.i64_const(instr.imm);
                self.op(OP_I64_ADD);
                self.local_set(WRITEBACK_LOCAL);
                true
            }
            3 => {
                self.i64_const(instr.imm);
                self.op(OP_I64_ADD);
                self.local_set(ADDR_LOCAL);
                self.local_get(ADDR_LOCAL);
                self.local_set(WRITEBACK_LOCAL);
                true
            }
            _ => {
                self.i64_const(instr.imm);
                self.op(OP_I64_ADD);
                self.local_set(ADDR_LOCAL);
                false
            }
        }
    }

    fn emit_register_offset(&mut self, instr: Instr) -> Option<()> {
        match instr.cond {
            0b010 => self.emit_read_reg(instr.rm, false),
            0b110 => {
                self.emit_read_reg(instr.rm, false);
                self.op(OP_I32_WRAP_I64);
                self.op(OP_I64_EXTEND_I32_S);
            }
            0b011 | 0b111 => self.emit_read_reg(instr.rm, true),
            _ => return None,
        }
        if instr.imm > 1 {
            return None;
        }
        if instr.imm == 1 {
            let shift = instr.size.trailing_zeros() as u64;
            if shift != 0 {
                self.i64_const(shift);
                self.op(OP_I64_SHL);
            }
        }
        Some(())
    }
}
