use super::opcodes::*;
use super::*;
use crate::constants::PAGE_OFFSET_MASK;

impl WasmExpr {
    pub(super) fn emit_instr(&mut self, instr: crate::arm64::Instr, pc: u64) -> bool {
        match instr.op {
            Opcode::Nop | Opcode::NopBarrier => true,
            Opcode::Movz | Opcode::Movn => {
                self.emit_write_reg_with(instr.rd, instr.sf, |this| this.i64_const(instr.imm));
                true
            }
            Opcode::Movk => {
                let shift = (instr.cond as u64) * 16;
                let mask = !(0xffffu64 << shift);
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_read_reg(instr.rd, instr.sf);
                    this.i64_const(mask);
                    this.op(OP_I64_AND);
                    this.i64_const(instr.imm);
                    this.op(OP_I64_OR);
                });
                true
            }
            Opcode::MovReg => {
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_read_reg(instr.rm, instr.sf);
                });
                true
            }
            Opcode::Sxtw => {
                self.emit_write_reg_with(instr.rd, true, |this| {
                    this.emit_read_reg(instr.rn, false);
                    this.op(OP_I32_WRAP_I64);
                    this.op(OP_I64_EXTEND_I32_S);
                });
                true
            }
            Opcode::AddImm | Opcode::SubImm => {
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
                true
            }
            Opcode::Add | Opcode::Sub => {
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
            Opcode::AndImm | Opcode::OrrImm | Opcode::EorImm => {
                let op = logical_opcode(instr.op);
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_read_reg(instr.rn, instr.sf);
                    this.i64_const(instr.imm);
                    this.op(op);
                });
                true
            }
            Opcode::AndReg | Opcode::OrrReg | Opcode::EorReg => {
                let shift_type = instr.cond & 3;
                if !can_emit_shift(shift_type, instr.imm, instr.sf) {
                    return false;
                }
                let invert = (instr.cond & 4) != 0;
                let op = logical_opcode(instr.op);
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_read_reg(instr.rn, instr.sf);
                    this.emit_read_shifted_reg(instr.rm, shift_type, instr.imm, instr.sf);
                    if invert {
                        this.i64_const(if instr.sf { u64::MAX } else { u32::MAX as u64 });
                        this.op(OP_I64_XOR);
                    }
                    this.op(op);
                });
                true
            }
            Opcode::Adr => {
                let target = (pc as i64 + instr.imm as i64) as u64;
                self.emit_write_reg_with(instr.rd, true, |this| this.i64_const(target));
                true
            }
            Opcode::Ubfm | Opcode::Sbfm => {
                if !helpers::can_emit_bitfield(instr) {
                    return false;
                }
                let signed = instr.op == Opcode::Sbfm;
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_bitfield_extract(instr, signed);
                });
                true
            }
            Opcode::Bfm => {
                if !helpers::can_emit_bitfield(instr) {
                    return false;
                }
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_bitfield_insert(instr);
                });
                true
            }
            Opcode::Adrp => {
                let page = pc & !PAGE_OFFSET_MASK;
                let target = (page as i64 + instr.imm as i64) as u64;
                self.emit_write_reg_with(instr.rd, true, |this| this.i64_const(target));
                true
            }
            _ => false,
        }
    }
}
