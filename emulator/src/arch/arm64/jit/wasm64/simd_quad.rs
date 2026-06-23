use super::memory_address::ADDR_LOCAL;
use super::*;

const JIT_STORE_QUAD_GUEST_FUNC_INDEX: u32 = 8;
const JIT_LOAD_QUAD_GUEST_FUNC_INDEX: u32 = 9;
const SIMD_WORD0_LOCAL: u32 = 3;
const SIMD_WORD1_LOCAL: u32 = 4;
const SIMD_WORD2_LOCAL: u32 = 5;
const SIMD_WORD3_LOCAL: u32 = 6;

impl WasmExpr {
    pub(super) fn emit_load_simd_q_pair(&mut self, reg1: u8, reg2: u8) {
        self.local_get(ADDR_LOCAL);
        self.i32_const(8);
        self.call_func(JIT_LOAD_QUAD_GUEST_FUNC_INDEX);
        self.local_set(SIMD_WORD3_LOCAL);
        self.local_set(SIMD_WORD2_LOCAL);
        self.local_set(SIMD_WORD1_LOCAL);
        self.local_set(SIMD_WORD0_LOCAL);
        self.write_simd_q_from_locals(reg1, SIMD_WORD0_LOCAL, SIMD_WORD1_LOCAL);
        self.write_simd_q_from_locals(reg2, SIMD_WORD2_LOCAL, SIMD_WORD3_LOCAL);
    }

    pub(super) fn emit_store_simd_q_pair(&mut self, reg1: u8, reg2: u8) {
        self.local_get(ADDR_LOCAL);
        self.i32_const(8);
        self.emit_read_simd_half(reg1, false);
        self.emit_read_simd_half(reg1, true);
        self.emit_read_simd_half(reg2, false);
        self.emit_read_simd_half(reg2, true);
        self.call_func(JIT_STORE_QUAD_GUEST_FUNC_INDEX);
    }

    fn write_simd_q_from_locals(&mut self, reg: u8, low_local: u32, high_local: u32) {
        self.emit_write_simd_half_with(reg, false, |this| this.local_get(low_local));
        self.emit_write_simd_half_with(reg, true, |this| this.local_get(high_local));
    }
}
