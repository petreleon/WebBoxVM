use super::opcodes::*;
use super::*;

const BYTE_LOW_BITS: u64 = 0x0101_0101_0101_0101;
const BYTE_HIGH_BITS: u64 = 0x8080_8080_8080_8080;
const LOCAL_CMP_VALUE: u32 = 1;

impl WasmExpr {
    pub(super) fn emit_simd_cmeq_zero(&mut self, instr: crate::arm64::Instr) -> bool {
        if instr.imm != 1 || !matches!(instr.size, 8 | 16) {
            return false;
        }
        self.emit_cmeq_zero_half(instr.rd, instr.rn, false);
        if instr.size == 16 {
            self.emit_cmeq_zero_half(instr.rd, instr.rn, true);
        } else {
            self.emit_write_simd_half_with(instr.rd, true, |this| this.i64_const(0));
        }
        true
    }

    pub(super) fn emit_simd_cmeq_reg(&mut self, instr: crate::arm64::Instr) -> bool {
        if instr.imm != 1 || !matches!(instr.size, 8 | 16) || instr.rm == 0xff {
            return false;
        }
        self.emit_cmeq_reg_half(instr.rd, instr.rn, instr.rm, false);
        if instr.size == 16 {
            self.emit_cmeq_reg_half(instr.rd, instr.rn, instr.rm, true);
        } else {
            self.emit_write_simd_half_with(instr.rd, true, |this| this.i64_const(0));
        }
        true
    }

    fn emit_cmeq_zero_half(&mut self, rd: u8, rn: u8, high: bool) {
        self.emit_write_simd_half_with(rd, high, |this| {
            this.emit_cmeq_byte_mask_from(|expr| expr.emit_read_simd_half(rn, high));
        });
    }

    fn emit_cmeq_reg_half(&mut self, rd: u8, rn: u8, rm: u8, high: bool) {
        self.emit_write_simd_half_with(rd, high, |this| {
            this.emit_cmeq_byte_mask_from(|expr| {
                expr.emit_read_simd_half(rn, high);
                expr.emit_read_simd_half(rm, high);
                expr.op(OP_I64_XOR);
            });
        });
    }

    fn emit_cmeq_byte_mask_from(&mut self, value: impl FnOnce(&mut Self)) {
        value(self);
        self.local_set(LOCAL_CMP_VALUE);
        self.local_get(LOCAL_CMP_VALUE);
        self.i64_const(BYTE_LOW_BITS);
        self.op(OP_I64_SUB);
        self.local_get(LOCAL_CMP_VALUE);
        self.i64_const(u64::MAX);
        self.op(OP_I64_XOR);
        self.op(OP_I64_AND);
        self.i64_const(BYTE_HIGH_BITS);
        self.op(OP_I64_AND);
        self.i64_const(7);
        self.op(OP_I64_SHR_U);
        self.i64_const(0xff);
        self.op(OP_I64_MUL);
    }
}
