use super::encoding::{encode_u32, encode_u64};
use super::helpers::simd_half_offset;
use super::opcodes::*;
use super::*;

const LOCAL_SHIFTED_REG: u32 = 4;
pub(super) struct WasmExpr {
    code: Vec<u8>,
    imports_helpers: bool,
}

impl WasmExpr {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            code: Vec::with_capacity(capacity),
            imports_helpers: false,
        }
    }

    pub(super) fn into_parts(self) -> (Vec<u8>, bool) {
        (self.code, self.imports_helpers)
    }

    pub(super) fn mark_import_helper(&mut self) {
        self.imports_helpers = true;
    }

    pub(super) fn emit_read_reg(&mut self, reg: u8, sf: bool) {
        if reg >= ZERO_REGISTER_INDEX {
            self.i64_const(0);
            return;
        }
        self.emit_load(reg_offset(reg));
        self.mask_32_if_needed(sf);
    }

    pub(super) fn emit_read_base(&mut self, reg: u8, sf: bool) {
        let offset = if reg >= SP_REGISTER_INDEX {
            JIT_STATE_SP_OFFSET
        } else {
            reg_offset(reg)
        };
        self.emit_load(offset);
        self.mask_32_if_needed(sf);
    }

    pub(super) fn emit_read_shifted_reg(&mut self, reg: u8, shift_type: u8, amount: u64, sf: bool) {
        self.emit_read_reg(reg, sf);
        if amount == 0 {
            return;
        }

        if !sf && shift_type == 2 {
            self.op(OP_I32_WRAP_I64);
            self.op(OP_I64_EXTEND_I32_S);
        }
        if !sf && shift_type == 3 {
            self.emit_rotr32_const(amount);
            return;
        }

        self.i64_const(amount);
        self.op(match shift_type {
            0 => OP_I64_SHL,
            1 => OP_I64_SHR_U,
            2 => OP_I64_SHR_S,
            3 => OP_I64_ROTR,
            _ => unreachable!(),
        });
    }

    fn emit_rotr32_const(&mut self, amount: u64) {
        self.local_set(LOCAL_SHIFTED_REG);
        self.local_get(LOCAL_SHIFTED_REG);
        self.i64_const(amount);
        self.op(OP_I64_SHR_U);
        self.local_get(LOCAL_SHIFTED_REG);
        self.i64_const((32 - amount) & 31);
        self.op(OP_I64_SHL);
        self.op(OP_I64_OR);
    }

    pub(super) fn emit_write_reg_with(&mut self, reg: u8, sf: bool, value: impl FnOnce(&mut Self)) {
        if reg >= ZERO_REGISTER_INDEX {
            return;
        }
        self.emit_store_with(reg_offset(reg), sf, value);
    }

    pub(super) fn emit_write_reg_sp_with(
        &mut self,
        reg: u8,
        sf: bool,
        value: impl FnOnce(&mut Self),
    ) {
        let offset = if reg >= SP_REGISTER_INDEX {
            JIT_STATE_SP_OFFSET
        } else {
            reg_offset(reg)
        };
        self.emit_store_with(offset, sf, value);
    }

    pub(super) fn emit_read_simd_half(&mut self, reg: u8, high: bool) {
        self.emit_load(simd_half_offset(reg, high));
    }

    pub(super) fn emit_write_simd_half_with(
        &mut self,
        reg: u8,
        high: bool,
        value: impl FnOnce(&mut Self),
    ) {
        self.emit_store_with(simd_half_offset(reg, high), true, value);
    }

    pub(super) fn emit_store_const(&mut self, offset: u64, value: u64) {
        self.emit_store_with(offset, true, |this| this.i64_const(value));
    }

    pub(super) fn emit_read_pstate(&mut self) {
        self.emit_load(JIT_STATE_PSTATE_OFFSET);
    }

    pub(super) fn emit_write_pstate_with(&mut self, value: impl FnOnce(&mut Self)) {
        self.emit_store_with(JIT_STATE_PSTATE_OFFSET, true, value);
    }

    pub(super) fn emit_write_sp_el0_with(&mut self, value: impl FnOnce(&mut Self)) {
        self.emit_store_with(JIT_STATE_SP_EL0_OFFSET, true, value);
    }

    pub(super) fn emit_write_pc_with(&mut self, value: impl FnOnce(&mut Self)) {
        self.emit_store_with(JIT_STATE_PC_OFFSET, true, value);
    }

    fn emit_store_with(&mut self, offset: u64, sf: bool, value: impl FnOnce(&mut Self)) {
        self.emit_addr(offset);
        value(self);
        self.mask_32_if_needed(sf);
        self.op(OP_I64_STORE);
        encode_u32(&mut self.code, 3);
        encode_u64(&mut self.code, 0);
    }

    fn emit_load(&mut self, offset: u64) {
        self.emit_addr(offset);
        self.op(OP_I64_LOAD);
        encode_u32(&mut self.code, 3);
        encode_u64(&mut self.code, 0);
    }

    fn emit_addr(&mut self, offset: u64) {
        self.op(OP_LOCAL_GET);
        encode_u32(&mut self.code, 0);
        if offset != 0 {
            self.i64_const(offset);
            self.op(OP_I64_ADD);
        }
    }

    pub(super) fn mask_32_if_needed(&mut self, sf: bool) {
        if !sf {
            self.i64_const(u32::MAX as u64);
            self.op(OP_I64_AND);
        }
    }

    pub(super) fn raw(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }
}
