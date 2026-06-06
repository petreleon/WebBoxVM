use super::opcodes::*;
use super::*;
use crate::arm64::{Instr, Opcode};

const JIT_LOAD_GUEST_FUNC_INDEX: u32 = 0;
const ADDR_LOCAL: u32 = 1;
const WRITEBACK_LOCAL: u32 = 2;
const VALUE_LOCAL: u32 = 3;

impl WasmExpr {
    pub(super) fn emit_memory_load(&mut self, instr: Instr) -> bool {
        if !matches!(instr.size, 1 | 2 | 4 | 8) {
            return false;
        }
        let Some(writeback) = self.emit_load_address(instr) else {
            return false;
        };

        self.local_get(ADDR_LOCAL);
        self.i32_const(instr.size as i32);
        self.call_func(JIT_LOAD_GUEST_FUNC_INDEX);
        if instr.op == Opcode::LdrSign {
            self.emit_load_sign_extend(instr.size);
        }
        self.local_set(VALUE_LOCAL);
        self.emit_write_reg_with(instr.rd, instr.sf, |this| this.local_get(VALUE_LOCAL));
        if writeback {
            self.emit_write_reg_sp_with(instr.rn, true, |this| {
                this.local_get(WRITEBACK_LOCAL);
            });
        }
        true
    }

    fn emit_load_address(&mut self, instr: Instr) -> Option<bool> {
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

    fn emit_load_sign_extend(&mut self, size: u8) {
        let shift = match size {
            1 => 56,
            2 => 48,
            4 => 32,
            _ => 0,
        };
        if shift == 0 {
            return;
        }
        self.i64_const(shift);
        self.op(OP_I64_SHL);
        self.i64_const(shift);
        self.op(OP_I64_SHR_S);
    }
}
