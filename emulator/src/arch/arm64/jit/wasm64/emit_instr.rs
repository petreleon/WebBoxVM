use super::opcodes::*;
use super::*;
use crate::constants::PAGE_OFFSET_MASK;

impl WasmExpr {
    pub(super) fn emit_instr(&mut self, instr: crate::arch::arm64::Instr, pc: u64) -> bool {
        match instr.op {
            Opcode::Nop | Opcode::NopBarrier => true,
            op if helpers::is_wasm_noop_alias(op) => true,
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
            Opcode::CmpImm => {
                self.emit_cmp_imm(instr);
                true
            }
            Opcode::AddsImm => {
                self.emit_adds_imm(instr);
                true
            }
            Opcode::Adds => self.emit_adds_reg(instr),
            Opcode::SubsImm => {
                self.emit_subs_imm(instr);
                true
            }
            Opcode::Subs => self.emit_subs_reg(instr),
            Opcode::Cmp => self.emit_cmp_reg(instr),
            Opcode::Ccmp | Opcode::Ccmn => {
                self.emit_cond_compare(instr);
                true
            }
            Opcode::Csel | Opcode::Csinc | Opcode::Csinv | Opcode::Csneg => {
                self.emit_cond_select(instr);
                true
            }
            Opcode::DaifSet | Opcode::DaifClr => {
                self.emit_daif_imm(instr);
                true
            }
            Opcode::Rev => {
                self.emit_rev(instr);
                true
            }
            Opcode::Clz | Opcode::Rbit => self.emit_bit_count(instr),
            Opcode::Extr => {
                self.emit_extract(instr);
                true
            }
            Opcode::AddImm | Opcode::SubImm => {
                self.emit_add_sub_imm(instr);
                true
            }
            Opcode::Add | Opcode::Sub => self.emit_add_sub_reg(instr),
            Opcode::AddExt | Opcode::SubExt => self.emit_add_sub_ext(instr),
            Opcode::Adc | Opcode::Adcs | Opcode::Sbc | Opcode::Sbcs => {
                self.emit_addsub_carry(instr);
                true
            }
            Opcode::Udiv => {
                self.emit_udiv(instr);
                true
            }
            Opcode::Umulh => self.emit_umulh(instr),
            Opcode::Madd | Opcode::Msub => self.emit_madd_msub(instr),
            Opcode::AndImm | Opcode::OrrImm | Opcode::EorImm => {
                let op = logical_opcode(instr.op);
                self.emit_write_reg_with(instr.rd, instr.sf, |this| {
                    this.emit_read_reg(instr.rn, instr.sf);
                    this.i64_const(instr.imm);
                    this.op(op);
                });
                true
            }
            Opcode::AndsImm => {
                self.emit_ands_imm(instr);
                true
            }
            Opcode::AndsReg => self.emit_ands_reg(instr),
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
            Opcode::Lslv | Opcode::Lsrv | Opcode::Asrv | Opcode::Rorv => {
                self.emit_variable_shift(instr);
                true
            }
            Opcode::Ldar | Opcode::Ldr | Opcode::LdrSign => self.emit_memory_load(instr),
            Opcode::Mrs => self.emit_mrs(instr),
            Opcode::Msr => self.emit_msr(instr),
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
            _ => self.emit_simd_instr(instr),
        }
    }
}
