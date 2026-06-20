use super::opcodes::*;
use super::*;

impl WasmExpr {
    pub(super) fn emit_add_sub_imm(&mut self, instr: crate::arch::arm64::Instr) {
        let op = if instr.op == Opcode::AddImm {
            OP_I64_ADD
        } else {
            OP_I64_SUB
        };
        self.emit_write_reg_sp_with(instr.rd, instr.sf, |this| {
            this.emit_read_base(instr.rn, instr.sf);
            this.i64_const(instr.imm);
            this.op(op);
        });
    }

    pub(super) fn emit_add_sub_reg(&mut self, instr: crate::arch::arm64::Instr) -> bool {
        if !can_emit_shift(instr.cond, instr.imm, instr.sf) {
            return false;
        }
        let op = if instr.op == Opcode::Add {
            OP_I64_ADD
        } else {
            OP_I64_SUB
        };
        self.emit_write_reg_with(instr.rd, instr.sf, |this| {
            this.emit_read_reg(instr.rn, instr.sf);
            this.emit_read_shifted_reg(instr.rm, instr.cond, instr.imm, instr.sf);
            this.op(op);
        });
        true
    }

    pub(super) fn emit_add_sub_ext(&mut self, instr: crate::arch::arm64::Instr) -> bool {
        if instr.cond > 7 || instr.imm > 4 {
            return false;
        }
        let op = if instr.op == Opcode::AddExt {
            OP_I64_ADD
        } else {
            OP_I64_SUB
        };
        self.emit_write_reg_sp_with(instr.rd, instr.sf, |this| {
            this.emit_read_base(instr.rn, instr.sf);
            this.emit_extended_reg(instr.rm, instr.cond, instr.imm, instr.sf);
            this.op(op);
        });
        true
    }

    pub(super) fn emit_extended_reg(&mut self, rm: u8, option: u8, shift: u64, sf: bool) {
        let read_sf = if option == 3 || option == 7 {
            sf
        } else {
            option >= 2
        };
        self.emit_read_reg(rm, read_sf);
        match option {
            0 => self.i64_const(0xff),
            1 => self.i64_const(0xffff),
            2 => self.i64_const(u32::MAX as u64),
            3 | 7 => {}
            4 => self.emit_sign_extend(8),
            5 => self.emit_sign_extend(16),
            6 => {
                self.op(OP_I32_WRAP_I64);
                self.op(OP_I64_EXTEND_I32_S);
            }
            _ => unreachable!(),
        }
        if option <= 2 {
            self.op(OP_I64_AND);
        }
        if shift != 0 {
            self.i64_const(shift);
            self.op(OP_I64_SHL);
        }
        self.mask_32_if_needed(sf);
    }

    fn emit_sign_extend(&mut self, bits: u64) {
        let shift = 64 - bits;
        self.i64_const(shift);
        self.op(OP_I64_SHL);
        self.i64_const(shift);
        self.op(OP_I64_SHR_S);
    }
}
