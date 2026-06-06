use super::opcodes::*;
use super::*;

const BYTE_LOW_BITS: u64 = 0x0101_0101_0101_0101;
const BYTE_HIGH_BITS: u64 = 0x8080_8080_8080_8080;

impl WasmExpr {
    pub(super) fn emit_simd_cmeq_zero(&mut self, instr: crate::arm64::Instr) -> bool {
        if instr.imm != 1 || instr.size != 16 {
            return false;
        }
        self.emit_cmeq_zero_half(instr.rd, instr.rn, false);
        self.emit_cmeq_zero_half(instr.rd, instr.rn, true);
        true
    }

    fn emit_cmeq_zero_half(&mut self, rd: u8, rn: u8, high: bool) {
        self.emit_write_simd_half_with(rd, high, |this| {
            this.emit_cmeq_zero_byte_mask(rn, high);
        });
    }

    fn emit_cmeq_zero_byte_mask(&mut self, rn: u8, high: bool) {
        self.emit_read_simd_half(rn, high);
        self.i64_const(BYTE_LOW_BITS);
        self.op(OP_I64_SUB);
        self.emit_read_simd_half(rn, high);
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
