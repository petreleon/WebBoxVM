use super::opcodes::*;
use super::*;

const LOCAL_BYTE_A: u32 = 1;
const LOCAL_BYTE_B: u32 = 2;
const LOCAL_ACC: u32 = 3;

impl WasmExpr {
    pub(super) fn emit_simd_umaxp(&mut self, instr: crate::arm64::Instr) -> bool {
        if instr.imm != 1 || instr.size != 16 {
            return false;
        }
        self.emit_pairwise_byte_umax_half(instr.rd, instr.rn, false);
        self.emit_pairwise_byte_umax_half(instr.rd, instr.rm, true);
        true
    }

    fn emit_pairwise_byte_umax_half(&mut self, rd: u8, src: u8, high: bool) {
        self.emit_write_simd_half_with(rd, high, |this| {
            this.emit_pairwise_byte_umax(src);
        });
    }

    fn emit_pairwise_byte_umax(&mut self, src: u8) {
        self.i64_const(0);
        self.local_set(LOCAL_ACC);

        for pair in 0..8 {
            self.emit_simd_byte(src, pair * 2);
            self.local_set(LOCAL_BYTE_A);
            self.emit_simd_byte(src, pair * 2 + 1);
            self.local_set(LOCAL_BYTE_B);
            self.local_get(LOCAL_ACC);
            self.emit_byte_max();
            if pair != 0 {
                self.i64_const(pair as u64 * 8);
                self.op(OP_I64_SHL);
            }
            self.op(OP_I64_OR);
            self.local_set(LOCAL_ACC);
        }

        self.local_get(LOCAL_ACC);
    }

    fn emit_byte_max(&mut self) {
        self.local_get(LOCAL_BYTE_A);
        self.local_get(LOCAL_BYTE_B);
        self.local_get(LOCAL_BYTE_A);
        self.local_get(LOCAL_BYTE_B);
        self.op(OP_I64_GE_U);
        self.op(OP_SELECT);
    }

    fn emit_simd_byte(&mut self, src: u8, byte: usize) {
        self.emit_read_simd_half(src, byte >= 8);
        let shift = (byte & 7) * 8;
        if shift != 0 {
            self.i64_const(shift as u64);
            self.op(OP_I64_SHR_U);
        }
        self.i64_const(0xff);
        self.op(OP_I64_AND);
    }
}
