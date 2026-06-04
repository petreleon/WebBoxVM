//! Instruction execution engine — mutates CPU and bus state for every ARM64 instruction.
//!
//! For each decoded instruction, this module:
//!   1. Reads source registers (handling XZR/WZR semantics)
//!   2. Performs the operation (ALU, load/store, branch, etc.)
//!   3. Writes the result to the destination register
//!   4. Increments PC and the cycle counter
//!   5. Checks for timer interrupt delivery

mod alu;
mod branch;
mod debug;
mod load_store;
mod system;

pub(super) use super::opcodes::{Instr, Opcode};
use alu::*;
use branch::{branch, branch_link, branch_link_reg, branch_reg, branch_target};
use load_store::{exec_atomic, exec_exclusive, exec_ldp_stp, exec_ldr_lit, exec_ldr_str};
use system::{exec_brk, exec_dc_zva, exec_eret, exec_msr, exec_svc};

use super::Armv8Cpu;
use super::helpers::{cond_taken, read_base, read_reg, write_reg, write_reg_sp};
use crate::bus::SystemBus;
use crate::constants::*;
use std::env;

/// Execute one decoded instruction, returning an error string if something goes wrong.
pub fn execute(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    match instr.op {
        Opcode::Add => write_reg_sp(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, instr.sf).wrapping_add(shifted_reg_val(
                cpu,
                instr.rm,
                instr.cond,
                instr.imm as u8,
                instr.sf,
            )),
            instr.sf,
        ),
        Opcode::Sub => write_reg_sp(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, instr.sf).wrapping_sub(shifted_reg_val(
                cpu,
                instr.rm,
                instr.cond,
                instr.imm as u8,
                instr.sf,
            )),
            instr.sf,
        ),
        Opcode::Adc | Opcode::Adcs | Opcode::Sbc | Opcode::Sbcs => exec_addsub_carry(cpu, instr),
        Opcode::Adds => {
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = add_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }
        Opcode::Subs => {
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = sub_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }

        Opcode::Movz => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::Movn => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::MovReg => write_reg(cpu, instr.rd, read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Sxtw => {
            let val = read_reg(cpu, instr.rn, false);
            write_reg(cpu, instr.rd, ((val as i32) as i64) as u64, true);
        }
        Opcode::Movk => {
            let hw = instr.cond as u64;
            let mask = !(0xFFFFu64 << (hw * 16));
            let old = read_reg(cpu, instr.rd, instr.sf);
            write_reg(cpu, instr.rd, (old & mask) | instr.imm, instr.sf);
        }

        Opcode::AddImm => write_reg_sp(
            cpu,
            instr.rd,
            read_base(cpu, instr.rn, instr.sf).wrapping_add(instr.imm),
            instr.sf,
        ),
        Opcode::SubImm => write_reg_sp(
            cpu,
            instr.rd,
            read_base(cpu, instr.rn, instr.sf).wrapping_sub(instr.imm),
            instr.sf,
        ),
        Opcode::AddsImm => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let val = add_flags(cpu, lhs, instr.imm, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }
        Opcode::SubsImm => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let val = sub_flags(cpu, lhs, instr.imm, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }
        Opcode::CmpImm => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let _ = sub_flags(cpu, lhs, instr.imm, instr.sf);
        }
        Opcode::Cmp => {
            let extended = (instr.cond & 0x8) != 0;
            let lhs = if extended {
                read_base(cpu, instr.rn, instr.sf)
            } else {
                read_reg(cpu, instr.rn, instr.sf)
            };
            let rhs = if extended {
                extend_reg_val(cpu, instr.rm, instr.cond & 0x7, instr.imm as u8, instr.sf)
            } else {
                shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)
            };
            let _ = sub_flags(cpu, lhs, rhs, instr.sf);
        }

        Opcode::Adr => write_reg(cpu, instr.rd, branch_target(cpu.regs.pc, instr.imm), true),
        Opcode::Adrp => {
            let page = cpu.regs.pc & !PAGE_OFFSET_MASK;
            write_reg(cpu, instr.rd, (page as i64 + instr.imm as i64) as u64, true);
        }

        // ── Load / Store ──
        Opcode::Ldr
        | Opcode::LdrSign
        | Opcode::Str
        | Opcode::SimdLdr
        | Opcode::SimdStr
        | Opcode::SimdLd1
        | Opcode::SimdLd1Multi
        | Opcode::SimdLd1Lane
        | Opcode::SimdLd1r
        | Opcode::SimdLd2
        | Opcode::SimdLd3
        | Opcode::SimdSt1Multi
        | Opcode::SimdSt1Lane
        | Opcode::SimdLd4
        | Opcode::SimdSt4Single
        | Opcode::SimdSt4 => exec_ldr_str(cpu, bus, instr)?,
        Opcode::LdrLit => exec_ldr_lit(cpu, bus, instr)?,
        Opcode::Ldp | Opcode::Ldpsw | Opcode::Stp | Opcode::SimdLdp | Opcode::SimdStp => {
            exec_ldp_stp(cpu, bus, instr)?
        }
        Opcode::Ldxr | Opcode::Ldar | Opcode::Stxr | Opcode::Stlr | Opcode::Ldxp | Opcode::Stxp => {
            exec_exclusive(cpu, bus, instr)?
        }
        Opcode::Atomic | Opcode::AtomicPair | Opcode::Cas | Opcode::Casp => {
            exec_atomic(cpu, bus, instr)?
        }

        // ── Branches ──
        Opcode::B => return branch(cpu, instr.imm),
        Opcode::Bl => return branch_link(cpu, instr.imm),
        Opcode::Blr => return branch_link_reg(cpu, instr.rn),
        Opcode::Br => return branch_reg(cpu, instr.rn),
        Opcode::Ret => return branch_reg(cpu, instr.rn),
        Opcode::Cbz => {
            if read_reg(cpu, instr.rd, instr.sf) == 0 {
                return branch(cpu, instr.imm);
            }
        }
        Opcode::Cbnz => {
            if read_reg(cpu, instr.rd, instr.sf) != 0 {
                return branch(cpu, instr.imm);
            }
        }
        Opcode::BCond => {
            if cond_taken(cpu, instr.cond) {
                return branch(cpu, instr.imm);
            }
        }
        Opcode::Tbz => {
            if (read_reg(cpu, instr.rd, instr.sf) >> (instr.cond as u64)) & 1 == 0 {
                return branch(cpu, instr.imm);
            }
        }
        Opcode::Tbnz => {
            if (read_reg(cpu, instr.rd, instr.sf) >> (instr.cond as u64)) & 1 != 0 {
                return branch(cpu, instr.imm);
            }
        }

        // ── Conditional select / compare ──
        Opcode::Csel => write_reg(
            cpu,
            instr.rd,
            if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                read_reg(cpu, instr.rm, instr.sf)
            },
            instr.sf,
        ),
        Opcode::Csinc => write_reg(
            cpu,
            instr.rd,
            if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                read_reg(cpu, instr.rm, instr.sf).wrapping_add(1)
            },
            instr.sf,
        ),
        Opcode::Csinv => write_reg(
            cpu,
            instr.rd,
            if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                !read_reg(cpu, instr.rm, instr.sf)
            },
            instr.sf,
        ),
        Opcode::Csneg => write_reg(
            cpu,
            instr.rd,
            if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                0u64.wrapping_sub(read_reg(cpu, instr.rm, instr.sf))
            },
            instr.sf,
        ),
        Opcode::Ccmp | Opcode::Ccmn => exec_condcmp(cpu, instr),

        // ── Logical (immediate) ──
        Opcode::AndImm => write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, instr.sf) & instr.imm,
            instr.sf,
        ),
        Opcode::OrrImm => write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, instr.sf) | instr.imm,
            instr.sf,
        ),
        Opcode::EorImm => write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, instr.sf) ^ instr.imm,
            instr.sf,
        ),
        Opcode::AndsImm => {
            let val = read_reg(cpu, instr.rn, instr.sf) & instr.imm;
            set_nz_flags(cpu, val, instr.sf);
            write_reg(cpu, instr.rd, val, instr.sf);
        }

        // ── Logical (register) ──
        Opcode::AndReg | Opcode::OrrReg | Opcode::EorReg | Opcode::AndsReg => {
            exec_logical_reg(cpu, instr)
        }

        // ── Bitfield ──
        Opcode::Sbfm | Opcode::Bfm | Opcode::Ubfm => exec_bitfield(cpu, instr),

        // ── Extended register arithmetic ──
        Opcode::AddExt => write_reg_sp(
            cpu,
            instr.rd,
            read_base(cpu, instr.rn, instr.sf).wrapping_add(extend_reg_val(
                cpu,
                instr.rm,
                instr.cond,
                instr.imm as u8,
                instr.sf,
            )),
            instr.sf,
        ),
        Opcode::SubExt => write_reg_sp(
            cpu,
            instr.rd,
            read_base(cpu, instr.rn, instr.sf).wrapping_sub(extend_reg_val(
                cpu,
                instr.rm,
                instr.cond,
                instr.imm as u8,
                instr.sf,
            )),
            instr.sf,
        ),
        Opcode::AddsExt => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = add_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }
        Opcode::SubsExt => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = sub_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }

        // ── Multiply / divide ──
        Opcode::Madd => exec_madd(cpu, instr),
        Opcode::Msub => exec_msub(cpu, instr),
        Opcode::Umulh => {
            let n = read_reg(cpu, instr.rn, true);
            let m = read_reg(cpu, instr.rm, true);
            write_reg(
                cpu,
                instr.rd,
                ((n as u128).wrapping_mul(m as u128) >> 64) as u64,
                true,
            );
        }
        Opcode::Smulh => {
            let n = read_reg(cpu, instr.rn, true) as i64;
            let m = read_reg(cpu, instr.rm, true) as i64;
            write_reg(
                cpu,
                instr.rd,
                ((n as i128).wrapping_mul(m as i128) >> 64) as u64,
                true,
            );
        }
        Opcode::Udiv => exec_div(cpu, instr, false),
        Opcode::Sdiv => exec_div(cpu, instr, true),

        // ── Variable shift ──
        Opcode::Lslv => exec_variable_shift(cpu, instr, ShiftDir::Left),
        Opcode::Lsrv => exec_variable_shift(cpu, instr, ShiftDir::Right),
        Opcode::Asrv => exec_variable_shift(cpu, instr, ShiftDir::ArithRight),
        Opcode::Rorv => exec_variable_shift(cpu, instr, ShiftDir::RotateRight),
        Opcode::Extr => exec_extract(cpu, instr),

        // ── Bit manipulation ──
        Opcode::Rev => exec_rev(cpu, instr),
        Opcode::Rev32 => {
            let val = read_reg(cpu, instr.rn, true);
            let low = (val as u32).swap_bytes() as u64;
            let high = ((val >> 32) as u32).swap_bytes() as u64;
            write_reg(cpu, instr.rd, (high << 32) | low, true);
        }
        Opcode::Rev16 => exec_rev16(cpu, instr),
        Opcode::Rbit => exec_rbit(cpu, instr),
        Opcode::Clz => exec_clz(cpu, instr),
        Opcode::Crc32 => exec_crc32(cpu, instr),

        // ── System ──
        Opcode::Mrs => {
            let sysreg_id = instr.imm as u16;
            let val = if sysreg_id == SYSREG_DAIF {
                cpu.pstate.daif()
            } else {
                cpu.sys.read_sys_reg(sysreg_id, cpu.pstate.el())
            };
            write_reg(cpu, instr.rd, val, true);
        }
        Opcode::Msr => exec_msr(cpu, instr),
        Opcode::SimdMovi => {
            cpu.simd[instr.rd as usize] = if instr.cond == 0 {
                simd_replicate_byte(instr.imm as u8) & simd_vector_mask(instr.size as usize)
            } else {
                simd_replicate_element(instr.imm as u128, instr.cond as usize, instr.size as usize)
            };
        }
        Opcode::SimdAese
        | Opcode::SimdAesd
        | Opcode::SimdAesmc
        | Opcode::SimdAesimc
        | Opcode::SimdPmull
        | Opcode::SimdSha1h
        | Opcode::SimdSha256Su0
        | Opcode::SimdSha512Su0
        | Opcode::SimdSm3Partw1
        | Opcode::SimdEor3
        | Opcode::SimdBcax
        | Opcode::SimdRax1
        | Opcode::SimdXar
        | Opcode::SimdDupByte
        | Opcode::SimdDupElem
        | Opcode::SimdFmovReg64
        | Opcode::SimdFmovGprToD
        | Opcode::SimdFmovGprToS
        | Opcode::SimdFmovDToGpr
        | Opcode::SimdFmovSToGpr
        | Opcode::SimdFmovLaneToGpr
        | Opcode::SimdUmov
        | Opcode::SimdSmov
        | Opcode::SimdInsGprLane
        | Opcode::SimdCmeqZero
        | Opcode::SimdCmgeZero
        | Opcode::SimdCmeqReg
        | Opcode::SimdCmhsReg
        | Opcode::SimdCmhiReg
        | Opcode::SimdShrn
        | Opcode::SimdAddhn
        | Opcode::SimdAddVec
        | Opcode::SimdSubVec
        | Opcode::SimdMulVec
        | Opcode::SimdMlaVec
        | Opcode::SimdAddp
        | Opcode::SimdAddv
        | Opcode::SimdUmaxv
        | Opcode::SimdExt
        | Opcode::SimdSmaxVec
        | Opcode::SimdUmaxVec
        | Opcode::SimdUminVec
        | Opcode::SimdUmaxp
        | Opcode::SimdUminp
        | Opcode::SimdCnt
        | Opcode::SimdCmtst
        | Opcode::SimdShlImm
        | Opcode::SimdSli
        | Opcode::SimdSri
        | Opcode::SimdSshr
        | Opcode::SimdUshr
        | Opcode::SimdUshl
        | Opcode::SimdXtn
        | Opcode::SimdRev64
        | Opcode::SimdRev32
        | Opcode::SimdNot
        | Opcode::SimdBsl
        | Opcode::SimdBit
        | Opcode::SimdBif
        | Opcode::SimdAnd
        | Opcode::SimdOrr
        | Opcode::SimdOrn
        | Opcode::SimdEor
        | Opcode::SimdInsElem
        | Opcode::SimdUzp1
        | Opcode::SimdTrn1
        | Opcode::SimdZip1
        | Opcode::SimdZip2
        | Opcode::SimdTbl
        | Opcode::SimdBic
        | Opcode::SimdBicImm
        | Opcode::SimdMvni
        | Opcode::SimdUshll
        | Opcode::SimdSshll
        | Opcode::SimdShll
        | Opcode::SimdSaddl
        | Opcode::SimdUsubl
        | Opcode::SimdSsubw
        | Opcode::SimdUmlal
        | Opcode::SimdUqsub
        | Opcode::SimdAbs
        | Opcode::SimdNeg
        | Opcode::SimdScvtf
        | Opcode::SimdFcvtzs
        | Opcode::SimdFcvtzu
        | Opcode::SimdFpAddVec
        | Opcode::SimdFpSubVec
        | Opcode::SimdFpMulVec
        | Opcode::SimdFpDivVec
        | Opcode::SimdFpAbd
        | Opcode::SimdFpNeg => exec_simd_data(cpu, instr),
        Opcode::FpAdd
        | Opcode::FpSub
        | Opcode::FpMul
        | Opcode::FpFnmul
        | Opcode::FpDiv
        | Opcode::FpMaxnm
        | Opcode::FpMinnm
        | Opcode::FpNeg
        | Opcode::FpAbs
        | Opcode::FpSqrt
        | Opcode::FpFcvt
        | Opcode::FpFrintm
        | Opcode::FpFrintn
        | Opcode::FpFrinta
        | Opcode::FpFrintx
        | Opcode::FpFrintz
        | Opcode::FpFrintp
        | Opcode::FpFrinti
        | Opcode::FpMovImm
        | Opcode::Fmadd
        | Opcode::Fmsub
        | Opcode::Fnmsub
        | Opcode::Scvtf
        | Opcode::Ucvtf
        | Opcode::Fcvtns
        | Opcode::Fcvtms
        | Opcode::Fcvtzs
        | Opcode::Fcvtzu
        | Opcode::Fcvtas
        | Opcode::Fcmp
        | Opcode::Fcmpe
        | Opcode::Fccmp
        | Opcode::Fccmpe
        | Opcode::Fcsel => exec_fp_scalar(cpu, instr),
        Opcode::Tlbi => {
            cpu.tlb.invalidate_all();
        }
        Opcode::DcZva => exec_dc_zva(cpu, bus, instr)?,
        Opcode::Svc => return exec_svc(cpu, instr.imm),
        Opcode::Eret => return exec_eret(cpu),
        Opcode::Brk => return exec_brk(cpu, bus, instr),
        Opcode::Nop | Opcode::NopBarrier => {
            if instr.cond == 1 {
                let bits = instr.imm as u8;
                if bits & 2 != 0 {
                    trace_daif(cpu, "daifset");
                    cpu.pstate = cpu.pstate.with_irq_masked(true);
                    trace_daif(cpu, "daifset ->");
                }
            } else if instr.cond == 2 {
                let bits = instr.imm as u8;
                if bits & 2 != 0 {
                    trace_daif(cpu, "daifclr");
                    cpu.pstate = cpu.pstate.with_irq_masked(false);
                    trace_daif(cpu, "daifclr ->");
                }
            } else if instr.cond == 3 {
                cpu.clear_exclusive();
            } else if instr.cond == 4 {
                if let Some(deadline) = cpu.sys.next_timer_deadline() {
                    if deadline > cpu.sys.cycle_count {
                        cpu.sys.cycle_count = deadline;
                    }
                }
            }
        }
        Opcode::Wfi => {
            if let Some(deadline) = cpu.sys.next_timer_deadline() {
                if deadline > cpu.sys.cycle_count {
                    cpu.sys.cycle_count = deadline;
                }
            }
        }
        Opcode::Wfe => {
            if let Some(deadline) = cpu.sys.next_timer_deadline() {
                if deadline > cpu.sys.cycle_count {
                    cpu.sys.cycle_count = deadline;
                }
            }
        }
    }

    advance_pc(cpu);
    check_timer_irq(cpu);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
//  Post-execution helpers
// ═══════════════════════════════════════════════════════════════════

/// Advance PC by 4 bytes and increment the cycle counter.
fn advance_pc(cpu: &mut Armv8Cpu) {
    cpu.regs.pc += INSTRUCTION_SIZE;
    cpu.sys.cycle_count = cpu.sys.cycle_count.wrapping_add(1);
}

/// Check if the physical timer has expired and deliver an IRQ if so.
fn check_timer_irq(cpu: &mut Armv8Cpu) {
    if cpu.sys.vbar_el1 == 0 {
        return;
    }

    if cpu.sys.cntv_expired() && cpu.sys.cntv_unmasked() {
        cpu.sys.irq_pending = true;
        cpu.sys.last_irq_id = VIRTUAL_TIMER_IRQ_ID;
    } else if cpu.sys.cntp_expired() && cpu.sys.cntp_unmasked() {
        cpu.sys.irq_pending = true;
        cpu.sys.last_irq_id = PHYSICAL_TIMER_IRQ_ID;
    }

    if cpu.sys.irq_pending && !cpu.pstate.irq_masked() {
        trace_daif(cpu, "irq exception");
        cpu.clear_exclusive();
        let from_lower_el = cpu.pstate.el() == 0;
        cpu.sys.spsr_el1 = cpu.pstate.to_u64();
        cpu.sys.elr_el1 = cpu.regs.pc;
        cpu.sys.esr_el1 = 0;

        cpu.enter_el1_exception(from_lower_el);
        cpu.regs.pc = cpu.sys.vbar_el1
            + if from_lower_el {
                VBAR_IRQ_LOWER_EL_AARCH64
            } else {
                VBAR_IRQ_CURRENT_EL
            };
        trace_daif(cpu, "irq exception ->");
    }
}

fn trace_daif(cpu: &Armv8Cpu, label: &str) {
    if env::var_os("WEBBOXVM_TRACE_DAIF").is_some() {
        eprintln!(
            "DAIF {label} pc=0x{:016x} pstate=0x{:x} irq_masked={}",
            cpu.regs.pc,
            cpu.pstate.to_u64(),
            cpu.pstate.irq_masked()
        );
    }
}

#[cfg(test)]
mod tests;
