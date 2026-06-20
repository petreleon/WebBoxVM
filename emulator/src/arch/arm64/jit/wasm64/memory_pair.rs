use super::memory_address::{ADDR_LOCAL, VALUE_LOCAL, WRITEBACK_LOCAL};
use super::opcodes::*;
use super::*;
use crate::arch::arm64::{Instr, Opcode};

const JIT_LOAD_GUEST_FUNC_INDEX: u32 = 0;
const JIT_STORE_GUEST_FUNC_INDEX: u32 = 1;
const PAIR_VALUE2_LOCAL: u32 = 4;

impl WasmExpr {
    pub(super) fn emit_memory_pair_load(&mut self, instr: Instr) -> bool {
        let size = pair_access_size(instr);
        if !matches!(size, 4 | 8) {
            return false;
        }
        let signed = instr.op == Opcode::Ldpsw;
        let dest_sf = signed || instr.sf;
        let writeback = self.emit_pair_address(instr);
        self.emit_pair_load_call(size, VALUE_LOCAL, 0, signed);
        self.emit_pair_load_call(size, PAIR_VALUE2_LOCAL, size as u64, signed);
        self.emit_write_reg_with(instr.rd, dest_sf, |this| {
            this.local_get(VALUE_LOCAL);
        });
        self.emit_write_reg_with(instr.rm, dest_sf, |this| {
            this.local_get(PAIR_VALUE2_LOCAL);
        });
        if writeback {
            self.emit_write_reg_sp_with(instr.rn, true, |this| {
                this.local_get(WRITEBACK_LOCAL);
            });
        }
        true
    }

    pub(super) fn emit_memory_pair_store(&mut self, instr: Instr) -> bool {
        let size = pair_access_size(instr);
        if !matches!(size, 4 | 8) {
            return false;
        }
        let writeback = self.emit_pair_address(instr);
        self.emit_read_reg(instr.rd, instr.sf);
        self.local_set(VALUE_LOCAL);
        self.emit_read_reg(instr.rm, instr.sf);
        self.local_set(PAIR_VALUE2_LOCAL);
        self.emit_pair_store_call(size, VALUE_LOCAL, 0);
        self.emit_pair_store_call(size, PAIR_VALUE2_LOCAL, size as u64);
        if writeback {
            self.emit_write_reg_sp_with(instr.rn, true, |this| {
                this.local_get(WRITEBACK_LOCAL);
            });
        }
        true
    }

    fn emit_pair_address(&mut self, instr: Instr) -> bool {
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

    fn emit_pair_store_call(&mut self, size: u8, value_local: u32, offset: u64) {
        self.local_get(ADDR_LOCAL);
        if offset != 0 {
            self.i64_const(offset);
            self.op(OP_I64_ADD);
        }
        self.i32_const(size as i32);
        self.local_get(value_local);
        self.call_func(JIT_STORE_GUEST_FUNC_INDEX);
    }

    fn emit_pair_load_call(&mut self, size: u8, target_local: u32, offset: u64, signed: bool) {
        self.local_get(ADDR_LOCAL);
        if offset != 0 {
            self.i64_const(offset);
            self.op(OP_I64_ADD);
        }
        self.i32_const(size as i32);
        self.call_func(JIT_LOAD_GUEST_FUNC_INDEX);
        if signed {
            self.i64_const(32);
            self.op(OP_I64_SHL);
            self.i64_const(32);
            self.op(OP_I64_SHR_S);
        }
        self.local_set(target_local);
    }
}

fn pair_access_size(instr: Instr) -> u8 {
    if instr.size != 0 {
        instr.size
    } else if instr.sf {
        8
    } else {
        4
    }
}
