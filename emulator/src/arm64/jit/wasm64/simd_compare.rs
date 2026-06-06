use super::opcodes::*;
use super::*;

const BYTE_LOW_BITS: u64 = 0x0101_0101_0101_0101;
const BYTE_HIGH_BITS: u64 = 0x8080_8080_8080_8080;
const LOCAL_CMP_VALUE: u32 = 1;
const LOCAL_CMP_ACC: u32 = 2;
const LOCAL_CMP_LHS: u32 = 3;

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

    pub(super) fn emit_simd_unsigned_cmp_reg(&mut self, instr: crate::arm64::Instr) -> bool {
        if instr.imm != 1 || !matches!(instr.size, 8 | 16) || instr.rm == 0xff {
            return false;
        }
        let or_equal = instr.op == Opcode::SimdCmhsReg;
        self.emit_unsigned_cmp_half(instr.rd, instr.rn, instr.rm, false, or_equal);
        if instr.size == 16 {
            self.emit_unsigned_cmp_half(instr.rd, instr.rn, instr.rm, true, or_equal);
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

    fn emit_unsigned_cmp_half(&mut self, rd: u8, rn: u8, rm: u8, high: bool, or_equal: bool) {
        self.emit_write_simd_half_with(rd, high, |this| {
            this.i64_const(0);
            this.local_set(LOCAL_CMP_ACC);
            for byte in 0..8 {
                this.local_get(LOCAL_CMP_ACC);
                this.emit_unsigned_cmp_byte(rn, rm, high, byte, or_equal);
                if byte != 0 {
                    this.i64_const(byte as u64 * 8);
                    this.op(OP_I64_SHL);
                }
                this.op(OP_I64_OR);
                this.local_set(LOCAL_CMP_ACC);
            }
            this.local_get(LOCAL_CMP_ACC);
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

    fn emit_unsigned_cmp_byte(&mut self, rn: u8, rm: u8, high: bool, byte: usize, or_equal: bool) {
        self.emit_simd_half_byte(rn, high, byte);
        self.local_set(LOCAL_CMP_LHS);
        self.i64_const(0xff);
        self.i64_const(0);
        if or_equal {
            self.local_get(LOCAL_CMP_LHS);
            self.emit_simd_half_byte(rm, high, byte);
            self.op(OP_I64_GE_U);
        } else {
            self.emit_simd_half_byte(rm, high, byte);
            self.local_get(LOCAL_CMP_LHS);
            self.op(OP_I64_LT_U);
        }
        self.op(OP_SELECT);
    }

    fn emit_simd_half_byte(&mut self, reg: u8, high: bool, byte: usize) {
        self.emit_read_simd_half(reg, high);
        if byte != 0 {
            self.i64_const(byte as u64 * 8);
            self.op(OP_I64_SHR_U);
        }
        self.i64_const(0xff);
        self.op(OP_I64_AND);
    }
}
