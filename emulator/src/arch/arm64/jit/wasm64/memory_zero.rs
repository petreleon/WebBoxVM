use super::memory_address::ADDR_LOCAL;
use super::opcodes::*;
use super::*;
use crate::arch::arm64::Instr;

const DC_ZVA_BLOCK_SIZE: u64 = 16;
const JIT_STORE_PAIR_GUEST_FUNC_INDEX: u32 = 6;

impl WasmExpr {
    pub(super) fn emit_dc_zva(&mut self, instr: Instr) {
        self.emit_read_reg(instr.rd, true);
        self.i64_const(!(DC_ZVA_BLOCK_SIZE - 1));
        self.op(OP_I64_AND);
        self.local_set(ADDR_LOCAL);
        self.local_get(ADDR_LOCAL);
        self.i32_const(8);
        self.i64_const(0);
        self.i64_const(0);
        self.call_func(JIT_STORE_PAIR_GUEST_FUNC_INDEX);
    }
}
