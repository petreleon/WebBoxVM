//! Execute decoded AArch64 instructions.

use super::opcodes::{Instr, Opcode};
use super::helpers::{cond_taken, read_reg, read_base, write_reg, write_reg_sp};
use super::Armv8Cpu;
use crate::bus::SystemBus;
use crate::arm64::mmu::translate;

/// Mutate CPU and bus state according to the decoded instruction.
pub fn execute(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    match instr.op {
        Opcode::Add  => write_reg_sp(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf).wrapping_add(shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::Sub  => write_reg_sp(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf).wrapping_sub(shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::Adds => {
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = add_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != 31 { write_reg_sp(cpu, instr.rd, val, instr.sf); }
        }
        Opcode::Subs => {
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = sub_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != 31 { write_reg_sp(cpu, instr.rd, val, instr.sf); }
        }
        Opcode::Movz => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::Movk => {
            // cond holds hw (0-3); imm holds imm16 << (hw*16)
            let hw = instr.cond as u64;
            let mask = !(0xFFFFu64 << (hw * 16));
            let old = read_reg(cpu, instr.rd, instr.sf);
            write_reg(cpu, instr.rd, (old & mask) | instr.imm, instr.sf);
        }

        Opcode::Movn => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::MovReg => write_reg(cpu, instr.rd, read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Sxtw => {
            let val = read_reg(cpu, instr.rn, false);
            let signed = ((val as i32) as i64) as u64;
            write_reg(cpu, instr.rd, signed, true);
        }
        Opcode::AddImm => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_add(instr.imm), instr.sf),
        Opcode::SubImm => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_sub(instr.imm), instr.sf),
        Opcode::AddsImm => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let val = add_flags(cpu, lhs, instr.imm, instr.sf);
            if instr.rd != 31 { write_reg_sp(cpu, instr.rd, val, instr.sf); }
        }
        Opcode::SubsImm => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let val = sub_flags(cpu, lhs, instr.imm, instr.sf);
            if instr.rd != 31 { write_reg_sp(cpu, instr.rd, val, instr.sf); }
        }
        Opcode::Adr    => write_reg(cpu, instr.rd, (cpu.regs.pc as i64 + instr.imm as i64) as u64, true),
        Opcode::Adrp   => {
            let page = cpu.regs.pc & !0xFFF;
            write_reg(cpu, instr.rd, (page as i64 + instr.imm as i64) as u64, true);
        }
        Opcode::Ldr => {
            let va = if instr.rm != 0xFF {
                // Register offset
                let base_addr = if instr.rn == 31 { cpu.regs.sp } else { cpu.regs.x(instr.rn) };
                let offset_val = read_reg(cpu, instr.rm, true);
                let extend_val = match instr.cond {
                    0b010 => (offset_val as u32) as u64, // UXTW
                    0b110 => (offset_val as i32) as i64 as u64, // SXTW
                    0b011 => offset_val, // LSL
                    0b111 => offset_val, // SXTX
                    _ => offset_val,
                };
                let shift = if instr.imm == 1 {
                    instr.size.trailing_zeros() as u8
                } else {
                    0
                };
                base_addr.wrapping_add(extend_val << shift)
            } else {
                // Immediate forms: cond=0 unsigned, cond=1 post-index, cond=3 pre-index
                let base_addr = if instr.rn == 31 { cpu.regs.sp } else { cpu.regs.x(instr.rn) };
                match instr.cond {
                    1 => base_addr, // Post-index: access base, write back after
                    3 => base_addr.wrapping_add(instr.imm), // Pre-index: access base+imm
                    _ => base_addr.wrapping_add(instr.imm), // Unsigned offset / unscaled
                }
            };
            let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDR translation fault")?;
            let size = if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 };
            let size = if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 };
            let val = bus.read(addr, size).ok_or("LDR bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
            // Writeback for pre/post-index
            if instr.rm == 0xFF {
                let base_addr = if instr.rn == 31 { cpu.regs.sp } else { cpu.regs.x(instr.rn) };
                let new_base = match instr.cond {
                    1 => Some(base_addr.wrapping_add(instr.imm)), // Post-index writeback
                    3 => Some(va),                                  // Pre-index writeback (= base+imm)
                    _ => None,
                };
                if let Some(nb) = new_base {
                    write_reg_sp(cpu, instr.rn, nb, true);
                }
            }
        }

        Opcode::LdrLit => {
            let va = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
            let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDR literal translation fault")?;
            let size = if instr.sf { 8 } else { 4 };
            let val = bus.read(addr, size).ok_or("LDR literal bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Str => {
            let va = if instr.rm != 0xFF {
                // Register offset
                let base_addr = if instr.rn == 31 { cpu.regs.sp } else { cpu.regs.x(instr.rn) };
                let offset_val = read_reg(cpu, instr.rm, true);
                let extend_val = match instr.cond {
                    0b010 => (offset_val as u32) as u64, // UXTW
                    0b110 => (offset_val as i32) as i64 as u64, // SXTW
                    0b011 => offset_val, // LSL
                    0b111 => offset_val, // SXTX
                    _ => offset_val,
                };
                let shift = if instr.imm == 1 {
                    instr.size.trailing_zeros() as u8
                } else {
                    0
                };
                base_addr.wrapping_add(extend_val << shift)
            } else {
                // Immediate forms: cond=0 unsigned, cond=1 post-index, cond=3 pre-index
                let base_addr = if instr.rn == 31 { cpu.regs.sp } else { cpu.regs.x(instr.rn) };
                match instr.cond {
                    1 => base_addr, // Post-index: store at base, write back after
                    3 => base_addr.wrapping_add(instr.imm), // Pre-index
                    _ => base_addr.wrapping_add(instr.imm), // Unsigned / unscaled
                }
            };
            let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "STR translation fault")?;
            let val = read_reg(cpu, instr.rd, instr.sf);
            let size = if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 };
            bus.write(addr, size, val);
            // Writeback for pre/post-index
            if instr.rm == 0xFF {
                let base_addr = if instr.rn == 31 { cpu.regs.sp } else { cpu.regs.x(instr.rn) };
                let new_base = match instr.cond {
                    1 => Some(base_addr.wrapping_add(instr.imm)), // Post-index
                    3 => Some(va),                                  // Pre-index
                    _ => None,
                };
                if let Some(nb) = new_base {
                    write_reg_sp(cpu, instr.rn, nb, true);
                }
            }
        }

        Opcode::Ldp => exec_ldp_stp(cpu, bus, instr, true, false)?,
        Opcode::Stp => exec_ldp_stp(cpu, bus, instr, false, false)?,
        Opcode::SimdLdp => exec_ldp_stp(cpu, bus, instr, true, true)?,
        Opcode::SimdStp => exec_ldp_stp(cpu, bus, instr, false, true)?,
        Opcode::Ldxr | Opcode::Ldar => {
            let base_addr = if instr.rn == 31 { cpu.regs.sp } else { cpu.regs.x(instr.rn) };
            let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base_addr).map_err(|_| "LDXR translation fault")?;
            let val = bus.read(addr, instr.size).ok_or("LDXR bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Ldxp => {
            let base_addr = if instr.rn == 31 { cpu.regs.sp } else { cpu.regs.x(instr.rn) };
            let size = if instr.sf { 8 } else { 4 };
            let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base_addr).map_err(|_| "LDXP translation fault")?;
            let addr2 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base_addr + size).map_err(|_| "LDXP translation fault")?;
            let val1 = bus.read(addr, size as u8).ok_or("LDXP bus fault")?;
            let val2 = bus.read(addr2, size as u8).ok_or("LDXP bus fault")?;
            write_reg(cpu, instr.rd, val1, instr.sf);
            write_reg(cpu, instr.rm, val2, instr.sf);
        }
        Opcode::Stxr | Opcode::Stlr => {
            let base_addr = if instr.rn == 31 { cpu.regs.sp } else { cpu.regs.x(instr.rn) };
            let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base_addr).map_err(|_| "STXR translation fault")?;
            let val = read_reg(cpu, instr.rd, instr.sf);
            bus.write(addr, instr.size, val);
            if instr.op == Opcode::Stxr {
                write_reg(cpu, instr.imm as u8, 0, false); // status register Ws
            }
        }
        Opcode::Stxp => {
            let base_addr = if instr.rn == 31 { cpu.regs.sp } else { cpu.regs.x(instr.rn) };
            let size = if instr.sf { 8 } else { 4 };
            let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base_addr).map_err(|_| "STXP translation fault")?;
            let addr2 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base_addr + size).map_err(|_| "STXP translation fault")?;
            let val1 = read_reg(cpu, instr.rd, instr.sf);
            let val2 = read_reg(cpu, instr.rm, instr.sf);
            bus.write(addr, size as u8, val1);
            bus.write(addr2, size as u8, val2);
            write_reg(cpu, instr.imm as u8, 0, false); // status register Ws
        }
        Opcode::B  => { cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64; return Ok(()); }
        Opcode::Bl => {
            cpu.regs.set_x(30, cpu.regs.pc + 4);
            cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
            return Ok(());
        }
        Opcode::Blr => {
            cpu.regs.set_x(30, cpu.regs.pc + 4);
            cpu.regs.pc = read_reg(cpu, instr.rn, true);
            return Ok(());
        }
        Opcode::Br  => { cpu.regs.pc = read_reg(cpu, instr.rn, true); return Ok(()); }
        Opcode::Ret => { cpu.regs.pc = read_reg(cpu, instr.rn, true); return Ok(()); }
        Opcode::Cbz  => {
            if read_reg(cpu, instr.rd, instr.sf) == 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Cbnz => {
            if read_reg(cpu, instr.rd, instr.sf) != 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::BCond => {
            if cond_taken(cpu, instr.cond) {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Csel => {
            let val = if cond_taken(cpu, instr.cond) { read_reg(cpu, instr.rn, instr.sf) } else { read_reg(cpu, instr.rm, instr.sf) };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Csinc => {
            let val = if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                read_reg(cpu, instr.rm, instr.sf).wrapping_add(1)
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Csinv => {
            let val = if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                !read_reg(cpu, instr.rm, instr.sf)
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Csneg => {
            let val = if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                0u64.wrapping_sub(read_reg(cpu, instr.rm, instr.sf))
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Ccmp => {
            if cond_taken(cpu, instr.cond) {
                let lhs = read_reg(cpu, instr.rn, instr.sf);
                let rhs = read_reg(cpu, instr.rm, instr.sf);
                let _ = sub_flags(cpu, lhs, rhs, instr.sf);
            } else {
                // CCMP nzcv field: n=bit3=8, z=bit2=4, c=bit1=2, v=bit0=1
                let n = (instr.imm & 8) != 0;
                let z = (instr.imm & 4) != 0;
                let c = (instr.imm & 2) != 0;
                let v = (instr.imm & 1) != 0;
                cpu.pstate.set_nzcv(n, z, c, v);
            }
        }
        Opcode::Tbz => {
            let val = read_reg(cpu, instr.rd, instr.sf);
            let bit = instr.cond as u64;
            if ((val >> bit) & 1) == 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Tbnz => {
            let val = read_reg(cpu, instr.rd, instr.sf);
            let bit = instr.cond as u64;
            if ((val >> bit) & 1) != 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Cmp => {
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            // cond >= 4 means extension type (SXTB=4, SXTH=5, SXTW=6, SXTX=7)
            // cond 0-3 are shift types for shifted-register form
            let rhs = if instr.cond >= 4 {
                extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)
            } else {
                shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)
            };
            let _ = sub_flags(cpu, lhs, rhs, instr.sf);
        }

        Opcode::CmpImm => {
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            let _ = sub_flags(cpu, lhs, instr.imm, instr.sf);
        }
        Opcode::Mrs => {
            let el = cpu.pstate.el();
            let sys_id = instr.imm as u16;
            let val = cpu.sys.read_sys_reg(sys_id, el);
            write_reg(cpu, instr.rd, val, true);
        }
        Opcode::Msr => {
            let val = read_reg(cpu, instr.rd, true);
            let sysreg_id = instr.imm as u16;
            cpu.sys.write_sys_reg(sysreg_id, val);
            match sysreg_id {
                0x4100 | 0x4101 | 0x4102 => cpu.tlb.invalidate_all(),
                _ => {}
            }
        }
        Opcode::Tlbi => {
            cpu.tlb.invalidate_all();
        }
        Opcode::Umulh => {
            let n = read_reg(cpu, instr.rn, true);
            let m = read_reg(cpu, instr.rm, true);
            let res = ((n as u128).wrapping_mul(m as u128) >> 64) as u64;
            write_reg(cpu, instr.rd, res, true);
        }
        Opcode::Smulh => {
            let n = read_reg(cpu, instr.rn, true) as i64;
            let m = read_reg(cpu, instr.rm, true) as i64;
            let res = ((n as i128).wrapping_mul(m as i128) >> 64) as u64;
            write_reg(cpu, instr.rd, res, true);
        }
        Opcode::Madd => {
            let sf_src = instr.size == 0 && instr.sf;
            let n = read_reg(cpu, instr.rn, sf_src);
            let m = read_reg(cpu, instr.rm, sf_src);
            let a = read_reg(cpu, instr.cond, instr.sf);

            let val = match instr.size {
                0 => {
                    if instr.sf {
                        a.wrapping_add(n.wrapping_mul(m))
                    } else {
                        let res = (a as u32).wrapping_add((n as u32).wrapping_mul(m as u32));
                        res as u64
                    }
                }
                1 => {
                    let prod = (n as u32 as u64).wrapping_mul(m as u32 as u64);
                    a.wrapping_add(prod)
                }
                2 => {
                    let n_signed = (n as u32 as i32) as i64;
                    let m_signed = (m as u32 as i32) as i64;
                    let prod = n_signed.wrapping_mul(m_signed) as u64;
                    a.wrapping_add(prod)
                }
                _ => return Err("Invalid Madd size"),
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Msub => {
            let sf_src = instr.size == 0 && instr.sf;
            let n = read_reg(cpu, instr.rn, sf_src);
            let m = read_reg(cpu, instr.rm, sf_src);
            let a = read_reg(cpu, instr.cond, instr.sf);

            let val = match instr.size {
                0 => {
                    if instr.sf {
                        a.wrapping_sub(n.wrapping_mul(m))
                    } else {
                        let res = (a as u32).wrapping_sub((n as u32).wrapping_mul(m as u32));
                        res as u64
                    }
                }
                1 => {
                    let prod = (n as u32 as u64).wrapping_mul(m as u32 as u64);
                    a.wrapping_sub(prod)
                }
                2 => {
                    let n_signed = (n as u32 as i32) as i64;
                    let m_signed = (m as u32 as i32) as i64;
                    let prod = n_signed.wrapping_mul(m_signed) as u64;
                    a.wrapping_sub(prod)
                }
                _ => return Err("Invalid Msub size"),
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::AndImm => {
            let val = read_reg(cpu, instr.rn, instr.sf) & instr.imm;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::OrrImm => {
            let val = read_reg(cpu, instr.rn, instr.sf) | instr.imm;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::EorImm => {
            let val = read_reg(cpu, instr.rn, instr.sf) ^ instr.imm;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::AndsImm => {
            let val = read_reg(cpu, instr.rn, instr.sf) & instr.imm;
            let sign_bit = if instr.sf { 63 } else { 31 };
            cpu.pstate.set_nzcv(((val >> sign_bit) & 1) != 0, if instr.sf { val == 0 } else { (val as u32) == 0 }, false, false);
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Sbfm | Opcode::Bfm | Opcode::Ubfm => {
            let size = if instr.sf { 64 } else { 32 };
            let r = instr.rm as u32; // immr
            let s = instr.imm as u32; // imms
            let src = read_reg(cpu, instr.rn, instr.sf);

            let val = if instr.op == Opcode::Ubfm {
                let result = if s >= r {
                    let len = s - r + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    (src >> r) & mask
                } else {
                    let len = s + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    let shift = size - r;
                    (src & mask) << shift
                };
                if !instr.sf { result & 0xFFFFFFFF } else { result }
            } else if instr.op == Opcode::Sbfm {
                let result = if s >= r {
                    let len = s - r + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    let extracted = (src >> r) & mask;
                    let sign_bit = s - r;
                    if sign_bit < 63 && (extracted & (1u64 << sign_bit)) != 0 {
                        let extend_mask = !((1u64 << (sign_bit + 1)) - 1);
                        extracted | (extend_mask & if instr.sf { !0 } else { 0xFFFFFFFF })
                    } else {
                        extracted
                    }
                } else {
                    let len = s + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    let shift = size - r;
                    let extracted = (src & mask) << shift;
                    let sign_bit = shift + s;
                    if sign_bit < 63 && (extracted & (1u64 << sign_bit)) != 0 {
                        let extend_mask = !((1u64 << (sign_bit + 1)) - 1);
                        extracted | (extend_mask & if instr.sf { !0 } else { 0xFFFFFFFF })
                    } else {
                        extracted
                    }
                };
                if !instr.sf { result & 0xFFFFFFFF } else { result }
            } else {
                let dst = read_reg(cpu, instr.rd, instr.sf);
                let result = if s >= r {
                    let len = s - r + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    let dst_mask = !(mask << r);
                    (dst & dst_mask) | ((src & mask) << r)
                } else {
                    let len = s + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    let shift = size - r;
                    let dst_mask = !(mask << shift);
                    (dst & dst_mask) | ((src & mask) << shift)
                };
                if !instr.sf { result & 0xFFFFFFFF } else { result }
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::AndReg | Opcode::OrrReg | Opcode::EorReg | Opcode::AndsReg => {
            let n = (instr.cond & 4) != 0;
            let shift_type = instr.cond & 3;
            let mut rhs = shifted_reg_val(cpu, instr.rm, shift_type, instr.imm as u8, instr.sf);
            if n {
                rhs = !rhs;
                if !instr.sf {
                    rhs &= 0xFFFFFFFF;
                }
            }
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            let val = match instr.op {
                Opcode::AndReg => lhs & rhs,
                Opcode::OrrReg => lhs | rhs,
                Opcode::EorReg => lhs ^ rhs,
                Opcode::AndsReg => {
                    let res = lhs & rhs;
                    let sign_bit = if instr.sf { 63 } else { 31 };
                    cpu.pstate.set_nzcv(((res >> sign_bit) & 1) != 0, if instr.sf { res == 0 } else { (res as u32) == 0 }, false, false);
                    res
                }
                _ => unreachable!(),
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::AddExt => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            write_reg_sp(cpu, instr.rd, lhs.wrapping_add(rhs), instr.sf);
        }
        Opcode::SubExt => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            write_reg_sp(cpu, instr.rd, lhs.wrapping_sub(rhs), instr.sf);
        }
        Opcode::AddsExt => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = add_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != 31 { write_reg_sp(cpu, instr.rd, val, instr.sf); }
        }
        Opcode::SubsExt => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = sub_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != 31 { write_reg_sp(cpu, instr.rd, val, instr.sf); }
        }
        Opcode::Nop | Opcode::NopBarrier => {}
        Opcode::Svc => {
            cpu.sys.elr_el1 = cpu.regs.pc + 4;
            cpu.sys.spsr_el1 = cpu.pstate.to_u64();
            cpu.pstate = cpu.pstate.with_el(1);
            cpu.regs.pc = cpu.sys.vbar_el1 + 0x400;
            return Ok(());
        }
        Opcode::Brk => {
            println!("BRK instruction hit at EL{}! imm16 = {:#x}, PC = {:#018x}", cpu.pstate.el(), instr.imm, cpu.regs.pc);
            println!("Registers: X0={:#018x} X1={:#018x} X2={:#018x} X3={:#018x}", cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2), cpu.regs.x(3));
            println!("           X4={:#018x} X5={:#018x} X6={:#018x} X7={:#018x}", cpu.regs.x(4), cpu.regs.x(5), cpu.regs.x(6), cpu.regs.x(7));
            println!("           X19={:#018x} X20={:#018x} X21={:#018x} X29={:#018x} LR={:#018x} SP={:#018x}", cpu.regs.x(19), cpu.regs.x(20), cpu.regs.x(21), cpu.regs.x(29), cpu.regs.x(30), cpu.regs.sp);
            println!("System Registers: VBAR_EL1={:#018x} ELR_EL1={:#018x} SPSR_EL1={:#018x}", cpu.sys.vbar_el1, cpu.sys.elr_el1, cpu.sys.spsr_el1);
            
            // Disassemble around PC
            println!("Instructions around PC ({:#018x}):", cpu.regs.pc);
            for offset in (-32..=32).step_by(4) {
                let addr = (cpu.regs.pc as i64 + offset) as u64;
                if let Ok(pa) = crate::arm64::mmu::translate(&cpu.sys, &mut cpu.tlb, &bus.mem, addr) {
                    if let Some(val) = bus.mem.read(pa, 4) {
                        let decoded = crate::arm64::decode(val as u32);
                        println!("  {:#018x}: {:08x} {:?}", addr, val, decoded.map(|d| d.op));
                    }
                }
            }

            // Disassemble around LR
            println!("Instructions around LR ({:#018x}):", cpu.regs.x(30));
            for offset in (-32..=32).step_by(4) {
                let addr = (cpu.regs.x(30) as i64 + offset) as u64;
                if let Ok(pa) = crate::arm64::mmu::translate(&cpu.sys, &mut cpu.tlb, &bus.mem, addr) {
                    if let Some(val) = bus.mem.read(pa, 4) {
                        let decoded = crate::arm64::decode(val as u32);
                        println!("  {:#018x}: {:08x} {:?}", addr, val, decoded.map(|d| d.op));
                    }
                }
            }

            // Inspect potential string pointers in X0, X1, X2, X3, X4
            for (i, &reg_val) in [cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2), cpu.regs.x(3), cpu.regs.x(4)].iter().enumerate() {
                if reg_val > 0xffff_8000_0000_0000 || (reg_val >= 0x4000_0000 && reg_val < 0x8000_0000) {
                    // Try to read it as a null-terminated ASCII string
                    let mut s = String::new();
                    let mut addr = reg_val;
                    let mut ok = true;
                    for _ in 0..128 {
                        if let Ok(pa) = crate::arm64::mmu::translate(&cpu.sys, &mut cpu.tlb, &bus.mem, addr) {
                            if let Some(b) = bus.mem.read(pa, 1) {
                                if b == 0 { break; }
                                if b >= 32 && b <= 126 || b == 10 || b == 13 {
                                    s.push(b as u8 as char);
                                    addr += 1;
                                } else {
                                    ok = false;
                                    break;
                                }
                            } else { ok = false; break; }
                        } else { ok = false; break; }
                    }
                    if ok && !s.is_empty() && s.len() > 2 {
                        println!("  X{} points to string: {:?}", i, s);
                    }

                    // Also print a raw hex dump of the first 64 bytes at this address
                    println!("  Raw memory dump at X{} ({:#018x}):", i, reg_val);
                    for offset in (0..64).step_by(16) {
                        let mut hex = String::new();
                        let mut ascii = String::new();
                        for j in 0..16 {
                            let curr_addr = reg_val + offset + j;
                            if let Ok(pa) = crate::arm64::mmu::translate(&cpu.sys, &mut cpu.tlb, &bus.mem, curr_addr) {
                                if let Some(b) = bus.mem.read(pa, 1) {
                                    hex.push_str(&format!("{:02x} ", b));
                                    if b >= 32 && b <= 126 {
                                        ascii.push(b as u8 as char);
                                    } else {
                                        ascii.push('.');
                                    }
                                } else {
                                    hex.push_str("?? ");
                                    ascii.push('.');
                                }
                            } else {
                                hex.push_str("?? ");
                                ascii.push('.');
                            }
                        }
                        println!("    +{:#04x}: {}  |{}|", offset, hex, ascii);
                    }
                }
            }

            println!("Stack (SP={:#018x}):", cpu.regs.sp);
            for offset in (0..128).step_by(16) {
                let addr = cpu.regs.sp + offset;
                let val1 = crate::arm64::mmu::translate(&cpu.sys, &mut cpu.tlb, &bus.mem, addr).ok().and_then(|pa| bus.mem.read(pa, 8)).unwrap_or(0);
                let val2 = crate::arm64::mmu::translate(&cpu.sys, &mut cpu.tlb, &bus.mem, addr + 8).ok().and_then(|pa| bus.mem.read(pa, 8)).unwrap_or(0);
                println!("  {:#018x}: {:#018x} {:#018x}", addr, val1, val2);
            }

            // Real AArch64 Exception Entry for BRK
            cpu.sys.elr_el1 = cpu.regs.pc;
            cpu.sys.spsr_el1 = cpu.pstate.to_u64();
            let esr = (0x3Cu64 << 26) | (instr.imm & 0xffff);
            cpu.sys.esr_el1 = esr;
            let pstate_el1 = cpu.pstate.with_el(1);
            let bits = pstate_el1.to_u64() | (0xF << 6);
            cpu.pstate = crate::arm64::pstate::ProcessorState::from_u64(bits);
            let target_pc = cpu.sys.vbar_el1 + 0x200;
            println!("Taking synchronous debug exception to VBAR_EL1 + 0x200 ({:#018x})", target_pc);
            cpu.regs.pc = target_pc;

            return Ok(());
        }
        Opcode::Eret => {
            cpu.regs.pc = cpu.sys.elr_el1;
            let spsr = cpu.sys.spsr_el1;
            cpu.pstate = crate::arm64::pstate::ProcessorState::from_u64(spsr);
            return Ok(());
        }
        Opcode::Rev => {
            if instr.sf {
                let val = read_reg(cpu, instr.rn, true);
                let res = val.swap_bytes();
                write_reg(cpu, instr.rd, res, true);
            } else {
                let val = read_reg(cpu, instr.rn, false) as u32;
                let res = val.swap_bytes() as u64;
                write_reg(cpu, instr.rd, res, false);
            }
        }
        Opcode::Rev32 => {
            let val = read_reg(cpu, instr.rn, true);
            let low = (val as u32).swap_bytes() as u64;
            let high = ((val >> 32) as u32).swap_bytes() as u64;
            let res = (high << 32) | low;
            write_reg(cpu, instr.rd, res, true);
        }
        Opcode::Rev16 => {
            if instr.sf {
                let val = read_reg(cpu, instr.rn, true);
                let res = ((val & 0xFF00FF00FF00FF00) >> 8) | ((val & 0x00FF00FF00FF00FF) << 8);
                write_reg(cpu, instr.rd, res, true);
            } else {
                let val = read_reg(cpu, instr.rn, false) as u32;
                let res = (((val & 0xFF00FF00) >> 8) | ((val & 0x00FF00FF) << 8)) as u64;
                write_reg(cpu, instr.rd, res, false);
            }
        }
        Opcode::Rbit => {
            if instr.sf {
                let val = read_reg(cpu, instr.rn, true);
                let res = val.reverse_bits();
                write_reg(cpu, instr.rd, res, true);
            } else {
                let val = read_reg(cpu, instr.rn, false) as u32;
                let res = val.reverse_bits() as u64;
                write_reg(cpu, instr.rd, res, false);
            }
        }
        Opcode::Clz => {
            if instr.sf {
                let val = read_reg(cpu, instr.rn, true);
                let res = val.leading_zeros() as u64;
                write_reg(cpu, instr.rd, res, true);
            } else {
                let val = read_reg(cpu, instr.rn, false) as u32;
                let res = val.leading_zeros() as u64;
                write_reg(cpu, instr.rd, res, false);
            }
        }
        Opcode::Udiv => {
            let n_val = read_reg(cpu, instr.rn, instr.sf);
            let m_val = read_reg(cpu, instr.rm, instr.sf);
            let res = if m_val == 0 {
                0
            } else if instr.sf {
                n_val / m_val
            } else {
                ((n_val as u32) / (m_val as u32)) as u64
            };
            write_reg(cpu, instr.rd, res, instr.sf);
        }
        Opcode::Sdiv => {
            let n_val = read_reg(cpu, instr.rn, instr.sf);
            let m_val = read_reg(cpu, instr.rm, instr.sf);
            let res = if m_val == 0 {
                0
            } else if instr.sf {
                let n = n_val as i64;
                let m = m_val as i64;
                n.checked_div(m).unwrap_or(n) as u64
            } else {
                let n = n_val as i32;
                let m = m_val as i32;
                n.checked_div(m).unwrap_or(n) as u32 as u64
            };
            write_reg(cpu, instr.rd, res, instr.sf);
        }
        Opcode::Lslv => {
            let n_val = read_reg(cpu, instr.rn, instr.sf);
            let m_val = read_reg(cpu, instr.rm, instr.sf);
            let res = if instr.sf {
                let shift = (m_val & 63) as u32;
                n_val << shift
            } else {
                let shift = (m_val & 31) as u32;
                ((n_val as u32) << shift) as u64
            };
            write_reg(cpu, instr.rd, res, instr.sf);
        }
        Opcode::Lsrv => {
            let n_val = read_reg(cpu, instr.rn, instr.sf);
            let m_val = read_reg(cpu, instr.rm, instr.sf);
            let res = if instr.sf {
                let shift = (m_val & 63) as u32;
                n_val >> shift
            } else {
                let shift = (m_val & 31) as u32;
                ((n_val as u32) >> shift) as u64
            };
            write_reg(cpu, instr.rd, res, instr.sf);
        }
        Opcode::Asrv => {
            let n_val = read_reg(cpu, instr.rn, instr.sf);
            let m_val = read_reg(cpu, instr.rm, instr.sf);
            let res = if instr.sf {
                let shift = (m_val & 63) as u32;
                ((n_val as i64) >> shift) as u64
            } else {
                let shift = (m_val & 31) as u32;
                ((n_val as i32) >> shift) as u32 as u64
            };
            write_reg(cpu, instr.rd, res, instr.sf);
        }
        Opcode::Rorv => {
            let n_val = read_reg(cpu, instr.rn, instr.sf);
            let m_val = read_reg(cpu, instr.rm, instr.sf);
            let res = if instr.sf {
                let shift = (m_val & 63) as u32;
                n_val.rotate_right(shift)
            } else {
                let shift = (m_val & 31) as u32;
                (n_val as u32).rotate_right(shift) as u64
            };
            write_reg(cpu, instr.rd, res, instr.sf);
        }
    }
    cpu.regs.pc += 4;
    cpu.sys.cycle_count = cpu.sys.cycle_count.wrapping_add(1);

    // Check timer: if enabled (bit0=1), not masked (bit2=0), and counter >= compare
    if (cpu.sys.cntp_ctl_el0 & 1) != 0 && (cpu.sys.cntp_ctl_el0 & 4) == 0 {
        if cpu.sys.cycle_count >= cpu.sys.cntp_cval_el0 {
            cpu.sys.irq_pending = true;
            cpu.sys.last_irq_id = 30; // PPI 30 = physical timer
        }
    }

    // Check for pending IRQ delivery
    if cpu.sys.irq_pending && !cpu.pstate.irq_masked() {
        // Save state
        cpu.sys.spsr_el1 = cpu.pstate.to_u64();
        cpu.sys.elr_el1 = cpu.regs.pc; // return to current instruction (retry)
        cpu.sys.esr_el1 = 0; // IRQ syndrome

        // Switch to EL1 with IRQ masked
        let pstate_el1 = cpu.pstate.with_el(1).with_irq_masked(true);
        // Set PSTATE bits: EL=1, IRQ masked, AArch64
        let spsr_bits = pstate_el1.to_u64() | (0xF << 6); // mask all DAIF
        cpu.pstate = crate::arm64::pstate::ProcessorState::from_u64(spsr_bits);

        // Jump to IRQ vector
        cpu.regs.pc = cpu.sys.vbar_el1 + 0x80;
    }

    Ok(())
}

fn extend_reg_val(cpu: &Armv8Cpu, rm: u8, option: u8, shift: u8, sf: bool) -> u64 {
    let mut val = read_reg(cpu, rm, if option == 3 || option == 7 { sf } else { option >= 2 });
    val = match option {
        0 => (val as u8) as u64,
        1 => (val as u16) as u64,
        2 => (val as u32) as u64,
        3 => val,
        4 => ((val as i8) as i64) as u64,
        5 => ((val as i16) as i64) as u64,
        6 => ((val as i32) as i64) as u64,
        7 => val,
        _ => val,
    };
    if sf {
        val << shift
    } else {
        ((val as u32) << shift) as u64
    }
}

fn add_flags(cpu: &mut Armv8Cpu, lhs: u64, rhs: u64, sf: bool) -> u64 {
    let val = lhs.wrapping_add(rhs);
    let sign_bit = if sf { 63 } else { 31 };
    let n = ((val >> sign_bit) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf {
        val < lhs
    } else {
        (val as u32) < (lhs as u32)
    };
    let sign_mask = 1u64 << sign_bit;
    let lhs_sign = lhs & sign_mask;
    let rhs_sign = rhs & sign_mask;
    let res_sign = val & sign_mask;
    let v = (lhs_sign == rhs_sign) && (lhs_sign != res_sign);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

fn sub_flags(cpu: &mut Armv8Cpu, lhs: u64, rhs: u64, sf: bool) -> u64 {
    let val = lhs.wrapping_sub(rhs);
    let sign_bit = if sf { 63 } else { 31 };
    let n = ((val >> sign_bit) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf { lhs >= rhs } else { (lhs as u32) >= (rhs as u32) };
    let sign_mask = 1u64 << sign_bit;
    let lhs_sign = lhs & sign_mask;
    let rhs_sign = rhs & sign_mask;
    let res_sign = val & sign_mask;
    let v = (lhs_sign != rhs_sign) && (lhs_sign != res_sign);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

fn shifted_reg_val(cpu: &Armv8Cpu, rm: u8, shift_type: u8, amount: u8, sf: bool) -> u64 {
    let val = read_reg(cpu, rm, sf);
    let amount = amount as u32;
    if amount == 0 {
        return val;
    }
    match shift_type {
        0 => { // LSL
            if sf { val << amount } else { ((val as u32) << amount) as u64 }
        }
        1 => { // LSR
            if sf { val >> amount } else { ((val as u32) >> amount) as u64 }
        }
        2 => { // ASR
            if sf {
                ((val as i64) >> amount) as u64
            } else {
                (((val as u32) as i32) >> amount) as u64
            }
        }
        3 => { // ROR
            if sf {
                val.rotate_right(amount)
            } else {
                (val as u32).rotate_right(amount) as u64
            }
        }
        _ => val,
    }
}

fn exec_ldp_stp(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr, is_load: bool, is_simd: bool) -> Result<(), &'static str> {
    let base = read_base(cpu, instr.rn, true);
    let size = if instr.size != 0 { instr.size as u64 } else if instr.sf { 8u64 } else { 4u64 };
    let (va, new_base) = match instr.cond {
        1 => (base, (base as i64).wrapping_add(instr.imm as i64) as u64),
        3 => {
            let b = (base as i64).wrapping_add(instr.imm as i64) as u64;
            (b, b)
        }
        _ => ((base as i64).wrapping_add(instr.imm as i64) as u64, base),
    };
    let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDP translation fault")?;
    let addr2 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va + size).map_err(|_| "LDP translation fault")?;
    if is_load {
        if !is_simd {
            write_reg(cpu, instr.rd, bus.read(addr, size as u8).ok_or("LDP bus fault")?, instr.sf);
            write_reg(cpu, instr.rm, bus.read(addr2, size as u8).ok_or("LDP bus fault")?, instr.sf);
        } else {
            // SIMD load: discard the result (we don't have V/D registers)
            let _ = bus.read(addr, size as u8).ok_or("LDP bus fault")?;
            let _ = bus.read(addr2, size as u8).ok_or("LDP bus fault")?;
        }
    } else {
        if !is_simd {
            bus.write(addr, size as u8, read_reg(cpu, instr.rd, instr.sf));
            bus.write(addr2, size as u8, read_reg(cpu, instr.rm, instr.sf));
        } else {
            // SIMD store: write zeros (we don't have V/D registers)
            bus.write(addr, size as u8, 0);
            bus.write(addr2, size as u8, 0);
        }
    }
    if new_base != base {
        write_reg_sp(cpu, instr.rn, new_base, true);
    }
    Ok(())
}

fn addr_with_offset(cpu: &Armv8Cpu, base: u8, offset: u64) -> Result<u64, &'static str> {
    let base_addr = if base >= 31 { cpu.regs.sp } else { cpu.regs.x(base) };
    Ok(base_addr.wrapping_add(offset))
}

#[cfg(test)]
mod tests;
