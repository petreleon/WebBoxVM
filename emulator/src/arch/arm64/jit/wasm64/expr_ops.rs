use super::WasmExpr;
use super::encoding::{encode_i64, encode_u32};
use super::opcodes::*;

impl WasmExpr {
    pub(super) fn local_get(&mut self, index: u32) {
        self.op(OP_LOCAL_GET);
        encode_u32(self.raw(), index);
    }

    pub(super) fn local_set(&mut self, index: u32) {
        self.op(OP_LOCAL_SET);
        encode_u32(self.raw(), index);
    }

    pub(super) fn call_func(&mut self, index: u32) {
        self.mark_import_helper();
        self.op(OP_CALL);
        encode_u32(self.raw(), index);
    }

    pub(super) fn i64_const(&mut self, value: u64) {
        self.op(OP_I64_CONST);
        encode_i64(self.raw(), value as i64);
    }

    pub(super) fn i32_const(&mut self, value: i32) {
        self.op(OP_I32_CONST);
        encode_i64(self.raw(), value as i64);
    }

    pub(super) fn op(&mut self, opcode: u8) {
        self.raw().push(opcode);
    }

    pub(super) fn end(&mut self) {
        self.op(OP_END);
    }
}
