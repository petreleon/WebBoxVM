use super::opcodes::*;
use super::*;

const LOCAL_NARROW_ACC: u32 = 1;

impl WasmExpr {
    pub(super) fn emit_simd_addhn(&mut self, instr: crate::arch::arm64::Instr) -> bool {
        let dest_bytes = instr.imm as usize;
        if !matches!(dest_bytes, 1 | 2 | 4) || instr.rm == 0xff {
            return false;
        }
        let high = matches!(instr.op, Opcode::SimdAddhn2 | Opcode::SimdRaddhn2);
        if high != (instr.size == 16) {
            return false;
        }
        let round = matches!(instr.op, Opcode::SimdRaddhn | Opcode::SimdRaddhn2);
        self.emit_write_simd_half_with(instr.rd, high, |this| {
            this.emit_narrow_add_half(instr.rn, instr.rm, dest_bytes, round);
        });
        if !high {
            self.emit_write_simd_half_with(instr.rd, true, |this| this.i64_const(0));
        }
        true
    }

    pub(super) fn emit_simd_shrn(&mut self, instr: crate::arch::arm64::Instr) -> bool {
        let dest_bytes = instr.cond as usize;
        if !matches!(dest_bytes, 1 | 2 | 4) || instr.imm == 0 || instr.imm >= 64 {
            return false;
        }
        let high = matches!(instr.op, Opcode::SimdShrn2 | Opcode::SimdRshrn2);
        if high != (instr.size == 16) {
            return false;
        }
        let round = matches!(instr.op, Opcode::SimdRshrn | Opcode::SimdRshrn2);
        self.emit_write_simd_half_with(instr.rd, high, |this| {
            this.emit_narrow_shift_half(instr.rn, dest_bytes, instr.imm as usize, round);
        });
        if !high {
            self.emit_write_simd_half_with(instr.rd, true, |this| this.i64_const(0));
        }
        true
    }

    fn emit_narrow_shift_half(&mut self, rn: u8, dest_bytes: usize, shift: usize, round: bool) {
        let lanes = 8 / dest_bytes;
        let dest_bits = dest_bytes * 8;
        self.i64_const(0);
        self.local_set(LOCAL_NARROW_ACC);
        for lane in 0..lanes {
            self.local_get(LOCAL_NARROW_ACC);
            self.emit_narrow_shifted_lane(rn, lane, dest_bytes, shift, round);
            if lane != 0 {
                self.i64_const((lane * dest_bits) as u64);
                self.op(OP_I64_SHL);
            }
            self.op(OP_I64_OR);
            self.local_set(LOCAL_NARROW_ACC);
        }
        self.local_get(LOCAL_NARROW_ACC);
    }

    fn emit_narrow_add_half(&mut self, rn: u8, rm: u8, dest_bytes: usize, round: bool) {
        let lanes = 8 / dest_bytes;
        let dest_bits = dest_bytes * 8;
        self.i64_const(0);
        self.local_set(LOCAL_NARROW_ACC);
        for lane in 0..lanes {
            self.local_get(LOCAL_NARROW_ACC);
            self.emit_narrow_added_lane(rn, rm, lane, dest_bytes, round);
            if lane != 0 {
                self.i64_const((lane * dest_bits) as u64);
                self.op(OP_I64_SHL);
            }
            self.op(OP_I64_OR);
            self.local_set(LOCAL_NARROW_ACC);
        }
        self.local_get(LOCAL_NARROW_ACC);
    }

    fn emit_narrow_added_lane(
        &mut self,
        rn: u8,
        rm: u8,
        lane: usize,
        dest_bytes: usize,
        round: bool,
    ) {
        let dest_bits = dest_bytes * 8;
        self.emit_source_element(rn, lane, dest_bytes * 2);
        self.emit_source_element(rm, lane, dest_bytes * 2);
        self.op(OP_I64_ADD);
        if round {
            self.i64_const(1u64 << (dest_bits - 1));
            self.op(OP_I64_ADD);
        }
        self.i64_const(dest_bits as u64);
        self.op(OP_I64_SHR_U);
        self.i64_const(element_mask(dest_bytes));
        self.op(OP_I64_AND);
    }

    fn emit_narrow_shifted_lane(
        &mut self,
        rn: u8,
        lane: usize,
        dest_bytes: usize,
        shift: usize,
        round: bool,
    ) {
        self.emit_source_element(rn, lane, dest_bytes * 2);
        if round {
            self.i64_const(1u64 << (shift - 1));
            self.op(OP_I64_ADD);
        }
        self.i64_const(shift as u64);
        self.op(OP_I64_SHR_U);
        self.i64_const(element_mask(dest_bytes));
        self.op(OP_I64_AND);
    }

    fn emit_source_element(&mut self, rn: u8, lane: usize, src_bytes: usize) {
        let byte_offset = lane * src_bytes;
        self.emit_read_simd_half(rn, byte_offset >= 8);
        let shift = (byte_offset & 7) * 8;
        if shift != 0 {
            self.i64_const(shift as u64);
            self.op(OP_I64_SHR_U);
        }
        self.i64_const(element_mask(src_bytes));
        self.op(OP_I64_AND);
    }
}

fn element_mask(bytes: usize) -> u64 {
    if bytes == 8 {
        u64::MAX
    } else {
        (1u64 << (bytes * 8)) - 1
    }
}
