use super::opcodes::OP_I64_AND;
use super::*;
use crate::arch::arm64::Instr;

impl WasmExpr {
    pub(super) fn emit_simd_movi(&mut self, instr: Instr) -> bool {
        let Some(value) = simd_replicated_value(instr) else {
            return false;
        };
        self.emit_write_simd_half_with(instr.rd, false, |this| this.i64_const(value as u64));
        self.emit_write_simd_half_with(instr.rd, true, |this| this.i64_const((value >> 64) as u64));
        true
    }

    pub(super) fn emit_simd_bic_imm(&mut self, instr: Instr) -> bool {
        let Some(mask) = simd_replicated_value(instr) else {
            return false;
        };
        self.emit_bic_imm_half(instr.rd, false, mask as u64);
        if instr.size == 16 {
            self.emit_bic_imm_half(instr.rd, true, (mask >> 64) as u64);
        } else {
            self.emit_write_simd_half_with(instr.rd, true, |this| this.i64_const(0));
        }
        true
    }

    fn emit_bic_imm_half(&mut self, rd: u8, high: bool, mask: u64) {
        self.emit_write_simd_half_with(rd, high, |this| {
            this.emit_read_simd_half(rd, high);
            this.i64_const(!mask);
            this.op(OP_I64_AND);
        });
    }
}

fn simd_replicated_value(instr: Instr) -> Option<u128> {
    let element_size = if instr.cond == 0 {
        1
    } else {
        instr.cond as usize
    };
    let vector_size = instr.size as usize;
    if !matches!(element_size, 1 | 2 | 4 | 8) || !matches!(vector_size, 8 | 16) {
        return None;
    }
    if vector_size % element_size != 0 {
        return None;
    }

    let bits = element_size * 8;
    let element = if bits == 64 {
        instr.imm as u128 & u64::MAX as u128
    } else {
        instr.imm as u128 & ((1u128 << bits) - 1)
    };
    let mut value = 0u128;
    for lane in 0..(vector_size / element_size) {
        value |= element << (lane * bits);
    }
    Some(value)
}
