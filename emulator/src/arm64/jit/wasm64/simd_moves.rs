use super::opcodes::*;
use super::*;
use crate::arm64::Instr;

impl WasmExpr {
    pub(super) fn emit_simd_dup_gpr(&mut self, instr: Instr) -> bool {
        let element_size = instr.cond.max(1);
        if !matches!(element_size, 1 | 2 | 4 | 8) || !matches!(instr.size, 8 | 16) {
            return false;
        }
        self.emit_dup_half(instr.rd, instr.rn, element_size, false);
        if instr.size == 16 {
            self.emit_dup_half(instr.rd, instr.rn, element_size, true);
        } else {
            self.emit_write_simd_half_with(instr.rd, true, |this| this.i64_const(0));
        }
        true
    }

    pub(super) fn emit_simd_fmov_d_to_gpr(&mut self, instr: Instr) -> bool {
        if instr.size != 8 {
            return false;
        }
        self.emit_write_reg_with(instr.rd, true, |this| {
            this.emit_read_simd_half(instr.rn, false);
        });
        true
    }

    fn emit_dup_half(&mut self, rd: u8, rn: u8, element_size: u8, high: bool) {
        self.emit_write_simd_half_with(rd, high, |this| {
            this.emit_dup_half_value(rn, element_size);
        });
    }

    fn emit_dup_half_value(&mut self, rn: u8, element_size: u8) {
        self.emit_read_reg(rn, element_size == 8);
        match element_size {
            1 => self.emit_replicated_gpr_element(0xff, 0x0101_0101_0101_0101),
            2 => self.emit_replicated_gpr_element(0xffff, 0x0001_0001_0001_0001),
            4 => {
                self.emit_read_reg(rn, false);
                self.i64_const(32);
                self.op(OP_I64_SHL);
                self.op(OP_I64_OR);
            }
            8 => {}
            _ => unreachable!(),
        }
    }

    fn emit_replicated_gpr_element(&mut self, mask: u64, multiplier: u64) {
        self.i64_const(mask);
        self.op(OP_I64_AND);
        self.i64_const(multiplier);
        self.op(OP_I64_MUL);
    }
}
