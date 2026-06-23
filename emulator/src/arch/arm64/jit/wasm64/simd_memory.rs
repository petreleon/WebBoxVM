use super::memory_address::{ADDR_LOCAL, WRITEBACK_LOCAL};
use super::opcodes::*;
use super::*;
use crate::arch::arm64::{Instr, Opcode};

const JIT_LOAD_GUEST_FUNC_INDEX: u32 = 0;
const JIT_LOAD_PAIR_GUEST_FUNC_INDEX: u32 = 7;
const SIMD_LOW_LOCAL: u32 = 3;
const SIMD_HIGH_LOCAL: u32 = 4;
const SIMD_POST_INDEX_IMM: u8 = 0xfe;

impl WasmExpr {
    pub(super) fn emit_simd_memory_load(&mut self, instr: Instr) -> bool {
        match instr.op {
            Opcode::SimdLd1 => self.emit_simd_ld1(instr),
            Opcode::SimdLd1Multi => self.emit_simd_ld1_multi(instr),
            Opcode::SimdLdp => self.emit_simd_ldp(instr),
            Opcode::SimdLdr => self.emit_simd_ldr(instr),
            Opcode::SimdStr => self.emit_simd_str(instr),
            _ => false,
        }
    }

    fn emit_simd_ld1(&mut self, instr: Instr) -> bool {
        if !matches!(instr.size, 8 | 16) || instr.cond != 1 {
            return false;
        }
        let writeback = self.emit_simd_structure_address(instr);
        if instr.size == 16 {
            self.emit_load_simd_q(instr.rd, 0);
        } else {
            self.emit_load_simd_half(instr.rd, false, 0);
            self.emit_write_simd_half_with(instr.rd, true, |this| this.i64_const(0));
        }
        if writeback {
            self.emit_write_reg_sp_with(instr.rn, true, |this| {
                this.local_get(WRITEBACK_LOCAL);
            });
        }
        true
    }

    fn emit_simd_ld1_multi(&mut self, instr: Instr) -> bool {
        if instr.size != 16 || instr.cond != 2 || instr.rm != 0xff {
            return false;
        }
        if self.emit_memory_address(instr) != Some(false) {
            return false;
        }
        self.emit_load_simd_q_pair(instr.rd, instr.rd.wrapping_add(1) & 31);
        true
    }

    fn emit_simd_ldp(&mut self, instr: Instr) -> bool {
        if instr.size != 16 {
            return false;
        }
        let writeback = self.emit_simd_pair_address(instr);
        self.emit_load_simd_q_pair(instr.rd, instr.rm);
        if writeback {
            self.emit_write_reg_sp_with(instr.rn, true, |this| {
                this.local_get(WRITEBACK_LOCAL);
            });
        }
        true
    }

    fn emit_simd_ldr(&mut self, instr: Instr) -> bool {
        if instr.size != 16 {
            return false;
        }
        let Some(writeback) = self.emit_memory_address(instr) else {
            return false;
        };
        self.emit_load_simd_q(instr.rd, 0);
        if writeback {
            self.emit_write_reg_sp_with(instr.rn, true, |this| {
                this.local_get(WRITEBACK_LOCAL);
            });
        }
        true
    }

    fn emit_simd_pair_address(&mut self, instr: Instr) -> bool {
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

    fn emit_load_simd_half(&mut self, reg: u8, high: bool, offset: u64) {
        self.emit_write_simd_half_with(reg, high, |this| {
            this.emit_guest_addr(offset);
            this.i32_const(8);
            this.call_func(JIT_LOAD_GUEST_FUNC_INDEX);
        });
    }

    fn emit_load_simd_q(&mut self, reg: u8, offset: u64) {
        self.emit_guest_addr(offset);
        self.i32_const(8);
        self.call_func(JIT_LOAD_PAIR_GUEST_FUNC_INDEX);
        self.local_set(SIMD_HIGH_LOCAL);
        self.local_set(SIMD_LOW_LOCAL);
        self.emit_write_simd_half_with(reg, false, |this| this.local_get(SIMD_LOW_LOCAL));
        self.emit_write_simd_half_with(reg, true, |this| this.local_get(SIMD_HIGH_LOCAL));
    }

    fn emit_guest_addr(&mut self, offset: u64) {
        self.local_get(ADDR_LOCAL);
        if offset != 0 {
            self.i64_const(offset);
            self.op(OP_I64_ADD);
        }
    }

    fn emit_simd_structure_address(&mut self, instr: Instr) -> bool {
        self.emit_read_base(instr.rn, true);
        self.local_set(ADDR_LOCAL);
        match instr.rm {
            0xff => false,
            SIMD_POST_INDEX_IMM => {
                self.local_get(ADDR_LOCAL);
                self.i64_const(instr.imm);
                self.op(OP_I64_ADD);
                self.local_set(WRITEBACK_LOCAL);
                true
            }
            rm => {
                self.local_get(ADDR_LOCAL);
                self.emit_read_reg(rm, true);
                self.op(OP_I64_ADD);
                self.local_set(WRITEBACK_LOCAL);
                true
            }
        }
    }
}
