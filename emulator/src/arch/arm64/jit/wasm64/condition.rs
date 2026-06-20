use super::opcodes::*;
use super::*;
use crate::constants::{PSTATE_C_BIT, PSTATE_N_BIT, PSTATE_V_BIT, PSTATE_Z_BIT};

impl WasmExpr {
    pub(super) fn emit_condition(&mut self, cond: u8) {
        match cond & 0xf {
            0b0000 => self.emit_flag_set(PSTATE_Z_BIT),
            0b0001 => self.emit_flag_clear(PSTATE_Z_BIT),
            0b0010 => self.emit_flag_set(PSTATE_C_BIT),
            0b0011 => self.emit_flag_clear(PSTATE_C_BIT),
            0b0100 => self.emit_flag_set(PSTATE_N_BIT),
            0b0101 => self.emit_flag_clear(PSTATE_N_BIT),
            0b0110 => self.emit_flag_set(PSTATE_V_BIT),
            0b0111 => self.emit_flag_clear(PSTATE_V_BIT),
            0b1000 => {
                self.emit_flag_set(PSTATE_C_BIT);
                self.emit_flag_clear(PSTATE_Z_BIT);
                self.op(OP_I32_AND);
            }
            0b1001 => {
                self.emit_flag_clear(PSTATE_C_BIT);
                self.emit_flag_set(PSTATE_Z_BIT);
                self.op(OP_I32_OR);
            }
            0b1010 => self.emit_n_equals_v(),
            0b1011 => self.emit_n_differs_v(),
            0b1100 => {
                self.emit_flag_clear(PSTATE_Z_BIT);
                self.emit_n_equals_v();
                self.op(OP_I32_AND);
            }
            0b1101 => {
                self.emit_flag_set(PSTATE_Z_BIT);
                self.emit_n_differs_v();
                self.op(OP_I32_OR);
            }
            _ => self.i32_const(1),
        }
    }

    fn emit_flag_set(&mut self, bit: u32) {
        self.emit_read_pstate();
        self.i64_const(1u64 << bit);
        self.op(OP_I64_AND);
        self.i64_const(0);
        self.op(OP_I64_NE);
    }

    fn emit_flag_clear(&mut self, bit: u32) {
        self.emit_read_pstate();
        self.i64_const(1u64 << bit);
        self.op(OP_I64_AND);
        self.op(OP_I64_EQZ);
    }

    fn emit_n_equals_v(&mut self) {
        self.emit_flag_set(PSTATE_N_BIT);
        self.emit_flag_set(PSTATE_V_BIT);
        self.op(OP_I32_EQ);
    }

    fn emit_n_differs_v(&mut self) {
        self.emit_flag_set(PSTATE_N_BIT);
        self.emit_flag_set(PSTATE_V_BIT);
        self.op(OP_I32_NE);
    }
}
