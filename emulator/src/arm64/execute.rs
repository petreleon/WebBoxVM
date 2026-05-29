//! Instruction execution engine — mutates CPU and bus state for every ARM64 instruction.
//!
//! For each decoded instruction, this module:
//!   1. Reads source registers (handling XZR/WZR semantics)
//!   2. Performs the operation (ALU, load/store, branch, etc.)
//!   3. Writes the result to the destination register
//!   4. Increments PC and the cycle counter
//!   5. Checks for timer interrupt delivery

use super::opcodes::{Instr, Opcode};
use super::helpers::{cond_taken, read_reg, read_base, write_reg, write_reg_sp};
use super::Armv8Cpu;
use crate::bus::SystemBus;
use crate::arm64::mmu::translate;
use crate::constants::*;

/// Execute one decoded instruction, returning an error string if something goes wrong.
pub fn execute(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    match instr.op {
        Opcode::Add  => write_reg_sp(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf).wrapping_add(shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::Sub  => write_reg_sp(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf).wrapping_sub(shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::Adds => { let lhs = read_reg(cpu, instr.rn, instr.sf); let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf); let val = add_flags(cpu, lhs, rhs, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }
        Opcode::Subs => { let lhs = read_reg(cpu, instr.rn, instr.sf); let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf); let val = sub_flags(cpu, lhs, rhs, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }

        Opcode::Movz  => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::Movn  => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::MovReg => write_reg(cpu, instr.rd, read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Sxtw  => { let val = read_reg(cpu, instr.rn, false); write_reg(cpu, instr.rd, ((val as i32) as i64) as u64, true); }
        Opcode::Movk  => { let hw = instr.cond as u64; let mask = !(0xFFFFu64 << (hw * 16)); let old = read_reg(cpu, instr.rd, instr.sf); write_reg(cpu, instr.rd, (old & mask) | instr.imm, instr.sf); }

        Opcode::AddImm   => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_add(instr.imm), instr.sf),
        Opcode::SubImm   => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_sub(instr.imm), instr.sf),
        Opcode::AddsImm  => { let lhs = read_base(cpu, instr.rn, instr.sf); let val = add_flags(cpu, lhs, instr.imm, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }
        Opcode::SubsImm  => { let lhs = read_base(cpu, instr.rn, instr.sf); let val = sub_flags(cpu, lhs, instr.imm, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }
        Opcode::CmpImm   => { let lhs = read_reg(cpu, instr.rn, instr.sf); let _ = sub_flags(cpu, lhs, instr.imm, instr.sf); }
        Opcode::Cmp      => { let lhs = read_reg(cpu, instr.rn, instr.sf); let rhs = ext_or_shifted_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf); let _ = sub_flags(cpu, lhs, rhs, instr.sf); }

        Opcode::Adr  => write_reg(cpu, instr.rd, branch_target(cpu.regs.pc, instr.imm), true),
        Opcode::Adrp => { let page = cpu.regs.pc & !PAGE_OFFSET_MASK; write_reg(cpu, instr.rd, (page as i64 + instr.imm as i64) as u64, true); }

        // ── Load / Store ──
        Opcode::Ldr | Opcode::Str => exec_ldr_str(cpu, bus, instr)?,
        Opcode::LdrLit            => exec_ldr_lit(cpu, bus, instr)?,
        Opcode::Ldp | Opcode::Stp | Opcode::SimdLdp | Opcode::SimdStp => exec_ldp_stp(cpu, bus, instr)?,
        Opcode::Ldxr | Opcode::Ldar | Opcode::Stxr | Opcode::Stlr | Opcode::Ldxp | Opcode::Stxp => exec_exclusive(cpu, bus, instr)?,

        // ── Branches ──
        Opcode::B   => return branch(cpu, instr.imm),
        Opcode::Bl  => return branch_link(cpu, instr.imm),
        Opcode::Blr => return branch_link_reg(cpu, instr.rn),
        Opcode::Br  => return branch_reg(cpu, instr.rn),
        Opcode::Ret => return branch_reg(cpu, instr.rn),
        Opcode::Cbz  => { if read_reg(cpu, instr.rd, instr.sf) == 0 { return branch(cpu, instr.imm); } }
        Opcode::Cbnz => { if read_reg(cpu, instr.rd, instr.sf) != 0 { return branch(cpu, instr.imm); } }
        Opcode::BCond => { if cond_taken(cpu, instr.cond) { return branch(cpu, instr.imm); } }
        Opcode::Tbz  => { if (read_reg(cpu, instr.rd, instr.sf) >> (instr.cond as u64)) & 1 == 0 { return branch(cpu, instr.imm); } }
        Opcode::Tbnz => { if (read_reg(cpu, instr.rd, instr.sf) >> (instr.cond as u64)) & 1 != 0 { return branch(cpu, instr.imm); } }

        // ── Conditional select / compare ──
        Opcode::Csel  => write_reg(cpu, instr.rd, if cond_taken(cpu, instr.cond) { read_reg(cpu, instr.rn, instr.sf) } else { read_reg(cpu, instr.rm, instr.sf) }, instr.sf),
        Opcode::Csinc => write_reg(cpu, instr.rd, if cond_taken(cpu, instr.cond) { read_reg(cpu, instr.rn, instr.sf) } else { read_reg(cpu, instr.rm, instr.sf).wrapping_add(1) }, instr.sf),
        Opcode::Csinv => write_reg(cpu, instr.rd, if cond_taken(cpu, instr.cond) { read_reg(cpu, instr.rn, instr.sf) } else { !read_reg(cpu, instr.rm, instr.sf) }, instr.sf),
        Opcode::Csneg => write_reg(cpu, instr.rd, if cond_taken(cpu, instr.cond) { read_reg(cpu, instr.rn, instr.sf) } else { 0u64.wrapping_sub(read_reg(cpu, instr.rm, instr.sf)) }, instr.sf),
        Opcode::Ccmp  => exec_ccmp(cpu, instr),

        // ── Logical (immediate) ──
        Opcode::AndImm  => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) & instr.imm, instr.sf),
        Opcode::OrrImm  => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) | instr.imm, instr.sf),
        Opcode::EorImm  => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) ^ instr.imm, instr.sf),
        Opcode::AndsImm => { let val = read_reg(cpu, instr.rn, instr.sf) & instr.imm; set_nz_flags(cpu, val, instr.sf); write_reg(cpu, instr.rd, val, instr.sf); }

        // ── Logical (register) ──
        Opcode::AndReg | Opcode::OrrReg | Opcode::EorReg | Opcode::AndsReg => exec_logical_reg(cpu, instr),

        // ── Bitfield ──
        Opcode::Sbfm | Opcode::Bfm | Opcode::Ubfm => exec_bitfield(cpu, instr),

        // ── Extended register arithmetic ──
        Opcode::AddExt  => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_add(extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::SubExt  => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_sub(extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::AddsExt => { let lhs = read_base(cpu, instr.rn, instr.sf); let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf); let val = add_flags(cpu, lhs, rhs, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }
        Opcode::SubsExt => { let lhs = read_base(cpu, instr.rn, instr.sf); let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf); let val = sub_flags(cpu, lhs, rhs, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }

        // ── Multiply / divide ──
        Opcode::Madd  => exec_madd(cpu, instr),
        Opcode::Msub  => exec_msub(cpu, instr),
        Opcode::Umulh => { let n = read_reg(cpu, instr.rn, true); let m = read_reg(cpu, instr.rm, true); write_reg(cpu, instr.rd, ((n as u128).wrapping_mul(m as u128) >> 64) as u64, true); }
        Opcode::Smulh => { let n = read_reg(cpu, instr.rn, true) as i64; let m = read_reg(cpu, instr.rm, true) as i64; write_reg(cpu, instr.rd, ((n as i128).wrapping_mul(m as i128) >> 64) as u64, true); }
        Opcode::Udiv  => exec_div(cpu, instr, false),
        Opcode::Sdiv  => exec_div(cpu, instr, true),

        // ── Variable shift ──
        Opcode::Lslv => exec_variable_shift(cpu, instr, ShiftDir::Left),
        Opcode::Lsrv => exec_variable_shift(cpu, instr, ShiftDir::Right),
        Opcode::Asrv => exec_variable_shift(cpu, instr, ShiftDir::ArithRight),
        Opcode::Rorv => exec_variable_shift(cpu, instr, ShiftDir::RotateRight),

        // ── Bit manipulation ──
        Opcode::Rev  => exec_rev(cpu, instr),
        Opcode::Rev32 => { let val = read_reg(cpu, instr.rn, true); let low = (val as u32).swap_bytes() as u64; let high = ((val >> 32) as u32).swap_bytes() as u64; write_reg(cpu, instr.rd, (high << 32) | low, true); }
        Opcode::Rev16 => exec_rev16(cpu, instr),
        Opcode::Rbit => exec_rbit(cpu, instr),
        Opcode::Clz  => exec_clz(cpu, instr),

        // ── System ──
        Opcode::Mrs    => { let val = cpu.sys.read_sys_reg(instr.imm as u16, cpu.pstate.el()); write_reg(cpu, instr.rd, val, true); }
        Opcode::Msr    => exec_msr(cpu, instr),
        Opcode::Tlbi   => { cpu.tlb.invalidate_all(); }
        Opcode::Svc    => return exec_svc(cpu),
        Opcode::Eret   => return exec_eret(cpu),
        Opcode::Brk    => return exec_brk(cpu, bus, instr),
        Opcode::Nop | Opcode::NopBarrier => {},
        Opcode::Wfi => {
            // Fast-forward cycle count to next timer expiry so any
            // pending timer IRQ fires immediately.  The kernel uses
            // WFI in idle/spin loops waiting for timer ticks.
            if cpu.sys.cntp_cval_el0 > cpu.sys.cycle_count {
                cpu.sys.cycle_count = cpu.sys.cntp_cval_el0;
            }
        },
        Opcode::Wfe => {
            // WFE: like WFI but can also be woken by events.
            // We treat it the same as WFI for now.
            if cpu.sys.cntp_cval_el0 > cpu.sys.cycle_count {
                cpu.sys.cycle_count = cpu.sys.cntp_cval_el0;
            }
        },
    }

    advance_pc(cpu);
    check_timer_irq(cpu);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
//  Branch helpers
// ═══════════════════════════════════════════════════════════════════

fn branch_target(pc: u64, offset: u64) -> u64 {
    (pc as i64 + offset as i64) as u64
}

/// Simple relative branch (B, B.cond, CBZ, CBNZ, TBZ, TBNZ).
fn branch(cpu: &mut Armv8Cpu, offset: u64) -> Result<(), &'static str> {
    cpu.regs.pc = branch_target(cpu.regs.pc, offset);
    Ok(())
}

/// Branch with Link: save return address in X30 (LR), then branch.
fn branch_link(cpu: &mut Armv8Cpu, offset: u64) -> Result<(), &'static str> {
    cpu.regs.set_x(LINK_REGISTER_INDEX, cpu.regs.pc + INSTRUCTION_SIZE);
    cpu.regs.pc = branch_target(cpu.regs.pc, offset);
    Ok(())
}

/// Branch to register (BR).
fn branch_reg(cpu: &mut Armv8Cpu, rn: u8) -> Result<(), &'static str> {
    cpu.regs.pc = read_reg(cpu, rn, true);
    Ok(())
}

/// Branch with Link to register (BLR): save return address, then branch to register.
fn branch_link_reg(cpu: &mut Armv8Cpu, rn: u8) -> Result<(), &'static str> {
    cpu.regs.set_x(LINK_REGISTER_INDEX, cpu.regs.pc + INSTRUCTION_SIZE);
    cpu.regs.pc = read_reg(cpu, rn, true);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
//  Load / Store helpers
// ═══════════════════════════════════════════════════════════════════

/// Compute the effective virtual address for a LDR or STR instruction.
fn compute_ldst_va(cpu: &Armv8Cpu, instr: &Instr) -> (u64, Option<u64>) {
    if instr.rm != 0xFF {
        // Register offset form
        let base = base_addr(cpu, instr.rn);
        let offset_val = read_reg(cpu, instr.rm, true);
        let extended = apply_extension(offset_val, instr.cond);
        let shift = if instr.imm == 1 { instr.size.trailing_zeros() as u8 } else { 0 };
        (base.wrapping_add(extended << shift), None)
    } else {
        // Immediate form
        let base = base_addr(cpu, instr.rn);
        let (va, wb) = match instr.cond {
            1 => (base, Some(base.wrapping_add(instr.imm))),           // Post-index
            3 => { let b = base.wrapping_add(instr.imm); (b, Some(b)) }, // Pre-index
            _ => (base.wrapping_add(instr.imm), None),                 // Unsigned / unscaled
        };
        (va, wb)
    }
}

/// Get the base address for a load/store: SP when rn == 31, else X[rn].
fn base_addr(cpu: &Armv8Cpu, rn: u8) -> u64 {
    if rn == SP_REGISTER_INDEX { cpu.regs.sp } else { cpu.regs.x(rn) }
}

/// Apply ARM64 extension type to a register value.
fn apply_extension(val: u64, option: u8) -> u64 {
    match option {
        0b010 => (val as u32) as u64,            // UXTW
        0b110 => (val as i32) as i64 as u64,     // SXTW
        0b011 => val,                             // LSL
        0b111 => val,                             // SXTX
        _ => val,
    }
}

/// Determine the access size for a load/store, defaulting to 4 or 8 bytes.
fn ldst_size(instr: &Instr) -> u8 {
    if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 }
}

fn exec_ldr_str(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let (va, writeback) = compute_ldst_va(cpu, &instr);
    let size = ldst_size(&instr);
    let is_load = instr.op == Opcode::Ldr;

    let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDR/STR translation fault")?;
    if is_load {
        let val = bus.read(pa, size).ok_or("LDR bus fault")?;
        write_reg(cpu, instr.rd, val, instr.sf);
    } else {
        let val = read_reg(cpu, instr.rd, instr.sf);
        bus.write(pa, size, val);
    }
    if let Some(new_base) = writeback {
        write_reg_sp(cpu, instr.rn, new_base, true);
    }
    Ok(())
}

fn exec_ldr_lit(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let va = branch_target(cpu.regs.pc, instr.imm);
    let size = if instr.sf { 8 } else { 4 };
    let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDR literal translation fault")?;
    let val = bus.read(pa, size).ok_or("LDR literal bus fault")?;
    write_reg(cpu, instr.rd, val, instr.sf);
    Ok(())
}

fn exec_ldp_stp(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let base = read_base(cpu, instr.rn, true);
    let size = if instr.size != 0 { instr.size as u64 } else if instr.sf { 8u64 } else { 4u64 };
    let (va, new_base) = match instr.cond {
        1 => (base, branch_target(base, instr.imm)),               // Post-index
        3 => { let b = branch_target(base, instr.imm); (b, b) },   // Pre-index
        _ => (branch_target(base, instr.imm), base),                // Signed offset
    };
    let pa1 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDP/STP translation fault")?;
    let pa2 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va + size).map_err(|_| "LDP/STP translation fault")?;

    match instr.op {
        Opcode::Ldp  => { write_reg(cpu, instr.rd, bus.read(pa1, size as u8).ok_or("LDP bus fault")?, instr.sf); write_reg(cpu, instr.rm, bus.read(pa2, size as u8).ok_or("LDP bus fault")?, instr.sf); }
        Opcode::Stp  => { bus.write(pa1, size as u8, read_reg(cpu, instr.rd, instr.sf)); bus.write(pa2, size as u8, read_reg(cpu, instr.rm, instr.sf)); }
        Opcode::SimdLdp => { let _ = bus.read(pa1, size as u8); let _ = bus.read(pa2, size as u8); } // No V/D regs
        Opcode::SimdStp => { bus.write(pa1, size as u8, 0); bus.write(pa2, size as u8, 0); }
        _ => unreachable!(),
    }
    if new_base != base { write_reg_sp(cpu, instr.rn, new_base, true); }
    Ok(())
}

fn exec_exclusive(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let base = base_addr(cpu, instr.rn);
    match instr.op {
        Opcode::Ldxr | Opcode::Ldar => {
            let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base).map_err(|_| "LDXR translation fault")?;
            let val = bus.read(pa, instr.size).ok_or("LDXR bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Stxr | Opcode::Stlr => {
            let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base).map_err(|_| "STXR translation fault")?;
            bus.write(pa, instr.size, read_reg(cpu, instr.rd, instr.sf));
            if instr.op == Opcode::Stxr {
                write_reg(cpu, instr.imm as u8, 0, false); // success status
            }
        }
        Opcode::Ldxp => {
            let size = if instr.sf { 8 } else { 4 };
            let pa1 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base).map_err(|_| "LDXP fault")?;
            let pa2 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base + size).map_err(|_| "LDXP fault")?;
            write_reg(cpu, instr.rd, bus.read(pa1, size as u8).ok_or("LDXP fault")?, instr.sf);
            write_reg(cpu, instr.rm, bus.read(pa2, size as u8).ok_or("LDXP fault")?, instr.sf);
        }
        Opcode::Stxp => {
            let size = if instr.sf { 8 } else { 4 };
            let pa1 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base).map_err(|_| "STXP fault")?;
            let pa2 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base + size).map_err(|_| "STXP fault")?;
            bus.write(pa1, size as u8, read_reg(cpu, instr.rd, instr.sf));
            bus.write(pa2, size as u8, read_reg(cpu, instr.rm, instr.sf));
            write_reg(cpu, instr.imm as u8, 0, false);
        }
        _ => unreachable!(),
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
//  System instruction helpers
// ═══════════════════════════════════════════════════════════════════

fn exec_msr(cpu: &mut Armv8Cpu, instr: Instr) {
    let val = read_reg(cpu, instr.rd, true);
    let sysreg_id = instr.imm as u16;
    cpu.sys.write_sys_reg(sysreg_id, val);
    // Invalidate TLB on writes to TTBR0_EL1, TTBR1_EL1, or TCR_EL1
    match sysreg_id {
        SYSREG_TTBR0_EL1 | SYSREG_TTBR1_EL1 | SYSREG_TCR_EL1 => cpu.tlb.invalidate_all(),
        _ => {}
    }
}

fn exec_svc(cpu: &mut Armv8Cpu) -> Result<(), &'static str> {
    // Enter EL1 from a lower EL via synchronous exception
    cpu.sys.elr_el1 = cpu.regs.pc + INSTRUCTION_SIZE;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.pstate = cpu.pstate.with_el(1);
    cpu.regs.pc = cpu.sys.vbar_el1 + VBAR_SYNC_LOWER_EL_AARCH64;
    Ok(())
}

fn exec_eret(cpu: &mut Armv8Cpu) -> Result<(), &'static str> {
    cpu.regs.pc = cpu.sys.elr_el1;
    cpu.pstate = super::pstate::ProcessorState::from_u64(cpu.sys.spsr_el1);
    Ok(())
}

fn exec_brk(cpu: &mut Armv8Cpu, bus: &SystemBus, instr: Instr) -> Result<(), &'static str> {
    let el = cpu.pstate.el();
    let imm16 = instr.imm;
    let pc = cpu.regs.pc;

    eprintln!("--- BRK HIT ---");
    eprintln!("  PC={:#018x}  EL={}  imm16=0x{:x}", pc, el, imm16);
    eprintln!("  X0={:#018x}  X1={:#018x}  X2={:#018x}  X3={:#018x}", cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2), cpu.regs.x(3));
    eprintln!("  X4={:#018x}  X5={:#018x}  X6={:#018x}  X7={:#018x}", cpu.regs.x(4), cpu.regs.x(5), cpu.regs.x(6), cpu.regs.x(7));
    eprintln!("  X19={:#018x}  X20={:#018x}  X21={:#018x}  X29={:#018x}  LR={:#018x}  SP={:#018x}", cpu.regs.x(19), cpu.regs.x(20), cpu.regs.x(21), cpu.regs.x(29), cpu.regs.x(30), cpu.regs.sp);
    eprintln!("  VBAR_EL1={:#018x}  ELR_EL1={:#018x}  SPSR_EL1={:#018x}", cpu.sys.vbar_el1, cpu.sys.elr_el1, cpu.sys.spsr_el1);

    // Dump instructions around PC
    dump_instructions("PC", pc, cpu, bus);
    dump_instructions("LR", cpu.regs.x(LINK_REGISTER_INDEX), cpu, bus);
    dump_string_pointers(cpu, bus);
    dump_stack(cpu, bus);

    // Take a synchronous debug exception (like real AArch64 BRK)
    let esr = (0x3Cu64 << 26) | (imm16 & 0xffff);
    cpu.sys.elr_el1 = pc;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.sys.esr_el1 = esr;

    let pstate_el1 = cpu.pstate.with_el(1);
    let spsr_bits = pstate_el1.to_u64() | SPSR_M_MASK;
    cpu.pstate = super::pstate::ProcessorState::from_u64(spsr_bits);

    cpu.regs.pc = cpu.sys.vbar_el1 + VBAR_SYNC_CURRENT_EL;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
//  Debug dump helpers (BRK handler)
// ═══════════════════════════════════════════════════════════════════

fn dump_instructions(label: &str, addr: u64, cpu: &Armv8Cpu, bus: &SystemBus) {
    let mut scratch_tlb = super::Tlb::new();
    eprintln!("Instructions around {} ({:#018x}):", label, addr);
    for offset in (-32..=32).step_by(4) {
        let target = branch_target(addr, offset as u64);
        if let Ok(pa) = translate(&cpu.sys, &mut scratch_tlb, &bus.mem, target) {
            if let Some(val) = bus.mem.read(pa, 4) {
                let decoded = super::decode(val as u32);
                eprintln!("  {:#018x}: {:08x} {:?}", target, val, decoded.map(|d| d.op));
            }
        }
    }
}

fn dump_string_pointers(cpu: &Armv8Cpu, bus: &SystemBus) {
    for (i, &reg_val) in [cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2), cpu.regs.x(3), cpu.regs.x(4)].iter().enumerate() {
        if reg_val == 0 { continue; }
        let mut scratch_tlb = super::Tlb::new();
        if let Some(s) = try_read_string_at(bus, &mut scratch_tlb, &cpu.sys, reg_val) {
            if !s.is_empty() && s.len() > 2 {
                eprintln!("  X{} points to string: {:?}", i, s);
            }
        }
    }
}

fn try_read_string_at(bus: &SystemBus, tlb: &mut super::Tlb, sys: &super::SystemRegisters, addr: u64) -> Option<String> {
    let mut s = String::new();
    let mut cur = addr;
    for _ in 0..128 {
        let pa = translate(sys, tlb, &bus.mem, cur).ok()?;
        let b = bus.mem.read(pa, 1)? as u8;
        if b == 0 { break; }
        if b.is_ascii_graphic() || b == b' ' || b == b'\n' || b == b'\r' {
            s.push(b as char);
            cur += 1;
        } else {
            return None;
        }
    }
    Some(s)
}

fn dump_stack(cpu: &Armv8Cpu, bus: &SystemBus) {
    let mut scratch_tlb = super::Tlb::new();
    eprintln!("Stack (SP={:#018x}):", cpu.regs.sp);
    for offset in (0..128).step_by(16) {
        let a = cpu.regs.sp + offset;
        let v1 = translate(&cpu.sys, &mut scratch_tlb, &bus.mem, a).ok().and_then(|pa| bus.mem.read(pa, 8)).unwrap_or(0);
        let v2 = translate(&cpu.sys, &mut scratch_tlb, &bus.mem, a + 8).ok().and_then(|pa| bus.mem.read(pa, 8)).unwrap_or(0);
        eprintln!("  {:#018x}: {:#018x} {:#018x}", a, v1, v2);
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Data-processing helpers
// ═══════════════════════════════════════════════════════════════════

fn exec_logical_reg(cpu: &mut Armv8Cpu, instr: Instr) {
    let n = (instr.cond & 4) != 0;
    let shift_type = instr.cond & 3;
    let mut rhs = shifted_reg_val(cpu, instr.rm, shift_type, instr.imm as u8, instr.sf);
    if n { rhs = !rhs; if !instr.sf { rhs &= 0xFFFFFFFF; } }
    let lhs = read_reg(cpu, instr.rn, instr.sf);
    let val = match instr.op {
        Opcode::AndReg => lhs & rhs,
        Opcode::OrrReg => lhs | rhs,
        Opcode::EorReg => lhs ^ rhs,
        Opcode::AndsReg => { set_nz_flags(cpu, lhs & rhs, instr.sf); lhs & rhs }
        _ => unreachable!(),
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

fn exec_bitfield(cpu: &mut Armv8Cpu, instr: Instr) {
    let size = if instr.sf { 64 } else { 32 };
    let r = instr.rm as u32;  // immr
    let s = instr.imm as u32; // imms
    let src = read_reg(cpu, instr.rn, instr.sf);

    let val = match instr.op {
        Opcode::Ubfm => bitfield_extract(src, r, s, size, false),
        Opcode::Sbfm => bitfield_extract(src, r, s, size, true),
        Opcode::Bfm  => {
            let dst = read_reg(cpu, instr.rd, instr.sf);
            bitfield_insert(dst, src, r, s, size)
        }
        _ => unreachable!(),
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

fn bitfield_extract(src: u64, r: u32, s: u32, size: u32, signed: bool) -> u64 {
    let result = if s >= r {
        let len = s - r + 1;
        let mask = bitmask(len);
        let extracted = (src >> r) & mask;
        if signed {
            sign_extend(extracted, s - r, size)
        } else {
            extracted
        }
    } else {
        let len = s + 1;
        let mask = bitmask(len);
        let shift = size - r;
        let extracted = (src & mask) << shift;
        if signed {
            sign_extend(extracted, shift + s, size)
        } else {
            extracted
        }
    };
    word_truncate(result, size)
}

fn bitfield_insert(dst: u64, src: u64, r: u32, s: u32, size: u32) -> u64 {
    let result = if s >= r {
        let len = s - r + 1;
        let mask = bitmask(len);
        let dst_mask = !(mask << r);
        (dst & dst_mask) | ((src & mask) << r)
    } else {
        let len = s + 1;
        let mask = bitmask(len);
        let shift = size - r;
        let dst_mask = !(mask << shift);
        (dst & dst_mask) | ((src & mask) << shift)
    };
    word_truncate(result, size)
}

fn bitmask(len: u32) -> u64 {
    if len >= 64 { !0 } else { (1u64 << len) - 1 }
}

fn sign_extend(val: u64, sign_bit: u32, size: u32) -> u64 {
    if sign_bit < 63 && (val & (1u64 << sign_bit)) != 0 {
        let extend_mask = !((1u64 << (sign_bit + 1)) - 1);
        val | (extend_mask & full_width_mask(size))
    } else {
        val
    }
}

fn word_truncate(val: u64, size: u32) -> u64 {
    if size == 64 { val } else { val & WORD_MASK }
}

fn full_width_mask(size: u32) -> u64 {
    if size == 64 { !0 } else { WORD_MASK }
}

fn exec_ccmp(cpu: &mut Armv8Cpu, instr: Instr) {
    if cond_taken(cpu, instr.cond) {
        let lhs = read_reg(cpu, instr.rn, instr.sf);
        let rhs = read_reg(cpu, instr.rm, instr.sf);
        let _ = sub_flags(cpu, lhs, rhs, instr.sf);
    } else {
        // CCMP nzcv field: bit3=N, bit2=Z, bit1=C, bit0=V
        let n = (instr.imm & 8) != 0;
        let z = (instr.imm & 4) != 0;
        let c = (instr.imm & 2) != 0;
        let v = (instr.imm & 1) != 0;
        cpu.pstate.set_nzcv(n, z, c, v);
    }
}

fn exec_madd(cpu: &mut Armv8Cpu, instr: Instr) {
    let sf_src = instr.size == 0 && instr.sf;
    let n = read_reg(cpu, instr.rn, sf_src);
    let m = read_reg(cpu, instr.rm, sf_src);
    let a = read_reg(cpu, instr.cond, instr.sf);
    let val = match instr.size {
        0 => if instr.sf { a.wrapping_add(n.wrapping_mul(m)) } else { ((a as u32).wrapping_add((n as u32).wrapping_mul(m as u32))) as u64 },
        1 => a.wrapping_add((n as u32 as u64).wrapping_mul(m as u32 as u64)),                    // UMADDL
        2 => a.wrapping_add(((n as u32 as i32) as i64).wrapping_mul((m as u32 as i32) as i64) as u64), // SMADDL
        _ => return,
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

fn exec_msub(cpu: &mut Armv8Cpu, instr: Instr) {
    let sf_src = instr.size == 0 && instr.sf;
    let n = read_reg(cpu, instr.rn, sf_src);
    let m = read_reg(cpu, instr.rm, sf_src);
    let a = read_reg(cpu, instr.cond, instr.sf);
    let val = match instr.size {
        0 => if instr.sf { a.wrapping_sub(n.wrapping_mul(m)) } else { ((a as u32).wrapping_sub((n as u32).wrapping_mul(m as u32))) as u64 },
        1 => a.wrapping_sub((n as u32 as u64).wrapping_mul(m as u32 as u64)),
        2 => a.wrapping_sub(((n as u32 as i32) as i64).wrapping_mul((m as u32 as i32) as i64) as u64),
        _ => return,
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

#[derive(Copy, Clone)]
enum ShiftDir { Left, Right, ArithRight, RotateRight }

fn exec_variable_shift(cpu: &mut Armv8Cpu, instr: Instr, dir: ShiftDir) {
    let n_val = read_reg(cpu, instr.rn, instr.sf);
    let m_val = read_reg(cpu, instr.rm, instr.sf);
    let val = if instr.sf {
        let shift = (m_val & 63) as u32;
        match dir {
            ShiftDir::Left => n_val << shift,
            ShiftDir::Right => n_val >> shift,
            ShiftDir::ArithRight => ((n_val as i64) >> shift) as u64,
            ShiftDir::RotateRight => n_val.rotate_right(shift),
        }
    } else {
        let shift = (m_val & 31) as u32;
        match dir {
            ShiftDir::Left => ((n_val as u32) << shift) as u64,
            ShiftDir::Right => ((n_val as u32) >> shift) as u64,
            ShiftDir::ArithRight => ((n_val as i32) >> shift) as u32 as u64,
            ShiftDir::RotateRight => (n_val as u32).rotate_right(shift) as u64,
        }
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

fn exec_div(cpu: &mut Armv8Cpu, instr: Instr, signed: bool) {
    let n = read_reg(cpu, instr.rn, instr.sf);
    let m = read_reg(cpu, instr.rm, instr.sf);
    let val = if m == 0 {
        0
    } else if instr.sf {
        if signed { (n as i64).checked_div(m as i64).unwrap_or(n as i64) as u64 } else { n / m }
    } else {
        if signed { (n as i32).checked_div(m as i32).unwrap_or(n as i32) as u32 as u64 } else { ((n as u32) / (m as u32)) as u64 }
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

fn exec_rev(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, true).swap_bytes(), true);
    } else {
        write_reg(cpu, instr.rd, (read_reg(cpu, instr.rn, false) as u32).swap_bytes() as u64, false);
    }
}

fn exec_rev16(cpu: &mut Armv8Cpu, instr: Instr) {
    // Swap bytes within each 16-bit halfword
    const MASK_EVEN: u64 = 0xFF00_FF00_FF00_FF00;
    const MASK_ODD:  u64 = 0x00FF_00FF_00FF_00FF;
    if instr.sf {
        let val = read_reg(cpu, instr.rn, true);
        write_reg(cpu, instr.rd, ((val & MASK_EVEN) >> 8) | ((val & MASK_ODD) << 8), true);
    } else {
        let val = read_reg(cpu, instr.rn, false) as u32;
        write_reg(cpu, instr.rd, (((val & 0xFF00_FF00) >> 8) | ((val & 0x00FF_00FF) << 8)) as u64, false);
    }
}

fn exec_rbit(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, true).reverse_bits(), true);
    } else {
        write_reg(cpu, instr.rd, (read_reg(cpu, instr.rn, false) as u32).reverse_bits() as u64, false);
    }
}

fn exec_clz(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, true).leading_zeros() as u64, true);
    } else {
        write_reg(cpu, instr.rd, (read_reg(cpu, instr.rn, false) as u32).leading_zeros() as u64, false);
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Flag-setting helpers
// ═══════════════════════════════════════════════════════════════════

/// Compute the sign bit position for the current operation width.
fn sign_bit(sf: bool) -> u32 {
    if sf { SIGN_BIT_64 } else { SIGN_BIT_32 }
}

/// Set N and Z flags from a result value.
fn set_nz_flags(cpu: &mut Armv8Cpu, val: u64, sf: bool) {
    let sb = sign_bit(sf);
    let is_zero = if sf { val == 0 } else { (val as u32) == 0 };
    cpu.pstate.set_nzcv(((val >> sb) & 1) != 0, is_zero, false, false);
}

fn add_flags(cpu: &mut Armv8Cpu, lhs: u64, rhs: u64, sf: bool) -> u64 {
    let val = lhs.wrapping_add(rhs);
    let sb = sign_bit(sf);
    let n = ((val >> sb) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf { val < lhs } else { (val as u32) < (lhs as u32) };
    let sign_mask = 1u64 << sb;
    let v = (lhs & sign_mask) == (rhs & sign_mask) && (lhs & sign_mask) != (val & sign_mask);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

fn sub_flags(cpu: &mut Armv8Cpu, lhs: u64, rhs: u64, sf: bool) -> u64 {
    let val = lhs.wrapping_sub(rhs);
    let sb = sign_bit(sf);
    let n = ((val >> sb) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf { lhs >= rhs } else { (lhs as u32) >= (rhs as u32) };
    let sign_mask = 1u64 << sb;
    let v = (lhs & sign_mask) != (rhs & sign_mask) && (lhs & sign_mask) != (val & sign_mask);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

// ═══════════════════════════════════════════════════════════════════
//  Register extension & shifting
// ═══════════════════════════════════════════════════════════════════

/// Extend a register value according to the ARM64 extension option.
fn extend_reg_val(cpu: &Armv8Cpu, rm: u8, option: u8, shift: u8, sf: bool) -> u64 {
    let mut val = read_reg(cpu, rm, if option == 3 || option == 7 { sf } else { option >= 2 });
    val = match option {
        0 => (val as u8) as u64,              // UXTB
        1 => (val as u16) as u64,             // UXTH
        2 => (val as u32) as u64,             // UXTW
        3 => val,                              // UXTX (no change)
        4 => ((val as i8) as i64) as u64,     // SXTB
        5 => ((val as i16) as i64) as u64,    // SXTH
        6 => ((val as i32) as i64) as u64,    // SXTW
        7 => val,                              // SXTX (no change)
        _ => val,
    };
    if sf { val << shift } else { ((val as u32) << shift) as u64 }
}

fn shifted_reg_val(cpu: &Armv8Cpu, rm: u8, shift_type: u8, amount: u8, sf: bool) -> u64 {
    let val = read_reg(cpu, rm, sf);
    let amount = amount as u32;
    if amount == 0 { return val; }
    match shift_type {
        0 => if sf { val << amount } else { ((val as u32) << amount) as u64 },   // LSL
        1 => if sf { val >> amount } else { ((val as u32) >> amount) as u64 },   // LSR
        2 => if sf { ((val as i64) >> amount) as u64 } else { (((val as u32) as i32) >> amount) as u64 }, // ASR
        3 => if sf { val.rotate_right(amount) } else { (val as u32).rotate_right(amount) as u64 },        // ROR
        _ => val,
    }
}

/// Pick between extension (cond >= 4) or shift (cond 0–3) for CMP.
fn ext_or_shifted_val(cpu: &Armv8Cpu, rm: u8, cond: u8, amount: u8, sf: bool) -> u64 {
    if cond >= 4 {
        extend_reg_val(cpu, rm, cond, amount, sf)
    } else {
        shifted_reg_val(cpu, rm, cond, amount, sf)
    }
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
    if cpu.sys.vbar_el1 == 0 { return; }

    cpu.sys.cntp_ctl_el0 |= TIMER_CTL_ENABLE;
    if cpu.sys.cntp_cval_el0 == 0 {
        cpu.sys.cntp_cval_el0 = TIMER_FREQ_HZ / 100;
    }

    if cpu.sys.cycle_count >= cpu.sys.cntp_cval_el0 {
        cpu.sys.irq_pending = true;
        cpu.sys.last_irq_id = TIMER_IRQ_ID;
        cpu.sys.cntp_cval_el0 = cpu.sys.cycle_count + TIMER_FREQ_HZ / 100;
    }

    if cpu.sys.irq_pending && !cpu.pstate.irq_masked() {
        cpu.sys.irq_pending = false;
        cpu.sys.spsr_el1 = cpu.pstate.to_u64();
        cpu.sys.elr_el1 = cpu.regs.pc;
        cpu.sys.esr_el1 = 0;

        let pstate_el1 = cpu.pstate.with_el(1).with_irq_masked(true);
        let spsr_bits = pstate_el1.to_u64() | SPSR_M_MASK;
        cpu.pstate = super::pstate::ProcessorState::from_u64(spsr_bits);

        cpu.regs.pc = cpu.sys.vbar_el1 + VBAR_IRQ_CURRENT_EL;
    }
}

#[cfg(test)]
mod tests;
