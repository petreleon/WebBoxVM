//! Execute decoded AArch64 instructions.

use super::opcodes::{Instr, Opcode};
use super::helpers::{cond_taken, read_reg, read_base, write_reg};
use super::Armv8Cpu;
use crate::bus::SystemBus;

/// Mutate CPU and bus state according to the decoded instruction.
pub fn execute(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    match instr.op {
        Opcode::Add  => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) + read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Sub  => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) - read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Movz => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::Movk => {
            let hw = instr.imm.trailing_zeros() / 16;
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
        Opcode::AddImm => write_reg(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf) + instr.imm, instr.sf),
        Opcode::SubImm => write_reg(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf) - instr.imm, instr.sf),
        Opcode::Adr    => write_reg(cpu, instr.rd, (cpu.regs.pc as i64 + instr.imm as i64) as u64, true),
        Opcode::Adrp   => {
            let page = cpu.regs.pc & !0xFFF;
            write_reg(cpu, instr.rd, (page as i64 + instr.imm as i64) as u64, true);
        }
        Opcode::Ldr => {
            let addr = addr_with_offset(cpu, instr.rn, instr.imm)?;
            let size = if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 };
            let val = bus.read(addr, size).ok_or("LDR bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::LdrLit => {
            let addr = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
            let size = if instr.sf { 8 } else { 4 };
            let val = bus.read(addr, size).ok_or("LDR literal bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Str => {
            let addr = addr_with_offset(cpu, instr.rn, instr.imm)?;
            let val = read_reg(cpu, instr.rd, instr.sf);
            let size = if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 };
            bus.write(addr, size, val);
        }
        Opcode::Ldp => exec_ldp_stp(cpu, bus, instr, true)?,
        Opcode::Stp => exec_ldp_stp(cpu, bus, instr, false)?,
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
        Opcode::Ccmp => {
            if cond_taken(cpu, instr.cond) {
                let lhs = read_reg(cpu, instr.rn, instr.sf);
                let rhs = read_reg(cpu, instr.rm, instr.sf);
                let val = lhs.wrapping_sub(rhs);
                cpu.pstate.set_nzcv((val >> 63) & 1 != 0, val == 0, true, false);
            }
        }
        Opcode::Tbz => {
            let val = read_reg(cpu, instr.rd, true);
            let bit = instr.cond as u64;
            if ((val >> bit) & 1) == 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Tbnz => {
            let val = read_reg(cpu, instr.rd, true);
            let bit = instr.cond as u64;
            if ((val >> bit) & 1) != 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Cmp => {
            let val = read_reg(cpu, instr.rn, instr.sf).wrapping_sub(read_reg(cpu, instr.rm, instr.sf));
            cpu.pstate.set_nzcv((val >> 63) & 1 != 0, val == 0, true, false);
        }
        Opcode::CmpImm => {
            let val = read_reg(cpu, instr.rn, instr.sf).wrapping_sub(instr.imm);
            cpu.pstate.set_nzcv((val >> 63) & 1 != 0, val == 0, true, false);
        }
        Opcode::Nop | Opcode::NopBarrier => {}
    }
    cpu.regs.pc += 4;
    Ok(())
}

fn exec_ldp_stp(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr, is_load: bool) -> Result<(), &'static str> {
    let base = read_base(cpu, instr.rn, true);
    let size = if instr.size != 0 { instr.size as u64 } else if instr.sf { 8u64 } else { 4u64 };
    let (addr, new_base) = match instr.cond {
        1 => (base, (base as i64).wrapping_add(instr.imm as i64) as u64),
        3 => {
            let b = (base as i64).wrapping_add(instr.imm as i64) as u64;
            (b, b)
        }
        _ => ((base as i64).wrapping_add(instr.imm as i64) as u64, base),
    };
    if is_load {
        write_reg(cpu, instr.rd, bus.read(addr, size as u8).ok_or("LDP bus fault")?, instr.sf);
        write_reg(cpu, instr.rm, bus.read(addr + size, size as u8).ok_or("LDP bus fault")?, instr.sf);
    } else {
        bus.write(addr, size as u8, read_reg(cpu, instr.rd, instr.sf));
        bus.write(addr + size, size as u8, read_reg(cpu, instr.rm, instr.sf));
    }
    if new_base != base {
        write_reg(cpu, instr.rn, new_base, true);
    }
    Ok(())
}

fn addr_with_offset(cpu: &Armv8Cpu, base: u8, offset: u64) -> Result<u64, &'static str> {
    let base_addr = if base >= 31 { cpu.regs.sp } else { cpu.regs.x(base) };
    Ok(base_addr.wrapping_add(offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arm64::decode::decode;

    fn setup() -> (Armv8Cpu, SystemBus) {
        (Armv8Cpu::new(), SystemBus::new())
    }

    #[test]
    fn add_x0_x1_x2() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(1, 10);
        cpu.regs.set_x(2, 32);
        execute(&mut cpu, &mut bus, decode(0x9A02_0020).unwrap()).unwrap();
        assert_eq!(cpu.regs.x(0), 42);
    }

    #[test]
    fn sub_x0_x1_x2() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(1, 50);
        cpu.regs.set_x(2, 8);
        execute(&mut cpu, &mut bus, decode(0xDA02_0020).unwrap()).unwrap();
        assert_eq!(cpu.regs.x(0), 42);
    }

    #[test]
    fn nop_advances_pc() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.pc = 0x4000_0000;
        execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0004);
    }

    #[test]
    fn branch_forward_4_bytes() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.pc = 0x4000_0000;
        execute(&mut cpu, &mut bus, decode(0x1400_0002).unwrap()).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0008);
    }

    #[test]
    fn bl_sets_lr_and_jumps() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.pc = 0x4000_0000;
        execute(&mut cpu, &mut bus, decode(0x9400_0002).unwrap()).unwrap();
        assert_eq!(cpu.regs.x(30), 0x4000_0004);
        assert_eq!(cpu.regs.pc, 0x4000_0008);
    }

    #[test]
    fn ret_returns_to_lr() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(30, 0x4000_0100);
        execute(&mut cpu, &mut bus, decode(0xD65F03C0).unwrap()).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0100);
    }

    #[test]
    fn cbz_branches_when_zero() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.pc = 0x4000_0000;
        cpu.regs.set_x(0, 0);
        execute(&mut cpu, &mut bus, decode(0xB400_0040).unwrap()).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0008);
    }

    #[test]
    fn cbz_falls_through_when_nonzero() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.pc = 0x4000_0000;
        cpu.regs.set_x(0, 1);
        execute(&mut cpu, &mut bus, decode(0xB400_0040).unwrap()).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0004);
    }

    #[test]
    fn ldp_loads_pair() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(1, 0x4000_0000);
        bus.mem.write(0x4000_0000, 8, 0xDEAD_BEEF);
        bus.mem.write(0x4000_0008, 8, 0xCAFE_BABE);
        execute(&mut cpu, &mut bus, decode(0xA940_0C22).unwrap()).unwrap();
        assert_eq!(cpu.regs.x(2), 0xDEAD_BEEF);
        assert_eq!(cpu.regs.x(3), 0xCAFE_BABE);
    }

    #[test]
    fn mov_reg_copies_value() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(1, 0x1234_5678);
        execute(&mut cpu, &mut bus, decode(0xAA01_03E0).unwrap()).unwrap();
        assert_eq!(cpu.regs.x(0), 0x1234_5678);
    }

    #[test]
    fn add_imm_adds_constant() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(1, 10);
        execute(&mut cpu, &mut bus, decode(0x9100_0420).unwrap()).unwrap();
        assert_eq!(cpu.regs.x(0), 11);
    }

    #[test]
    fn movk_merges_value() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(0, 0xDEAD_BEEF_0000_0000);
        execute(&mut cpu, &mut bus, decode(0xF282_4680).unwrap()).unwrap();
        assert_eq!(cpu.regs.x(0), 0xDEAD_BEEF_0000_1234);
    }

    #[test]
    fn adrp_sets_page_relative() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.pc = 0x4000_0400;
        execute(&mut cpu, &mut bus, decode(0x9000_0000).unwrap()).unwrap();
        assert_eq!(cpu.regs.x(0), 0x4000_0000);
    }

    #[test]
    fn tbz_branches_when_bit_clear() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.pc = 0x4000_0000;
        cpu.regs.set_x(0, 0b110);
        execute(&mut cpu, &mut bus, decode(0x3600_0020).unwrap()).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0004);
    }

    #[test]
    fn cmp_sets_flags() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(2, 10);
        cpu.regs.set_x(3, 5);
        execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()).unwrap();
        assert!(!cpu.pstate.z());
        assert!(cpu.pstate.n());
    }

    #[test]
    fn cmp_equal_sets_z() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(2, 5);
        cpu.regs.set_x(3, 5);
        execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()).unwrap();
        assert!(cpu.pstate.z());
        assert!(!cpu.pstate.n());
    }

    #[test]
    fn cmp_less_than_sets_n() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(2, 3);
        cpu.regs.set_x(3, 10);
        execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()).unwrap();
        assert!(!cpu.pstate.n());
        assert!(!cpu.pstate.z());
    }

    #[test]
    fn str_wzr_sp_60() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.sp = 0x4000_0000;
        execute(&mut cpu, &mut bus, decode(0xB900_3FFF).unwrap()).unwrap();
        assert_eq!(bus.mem.read(0x4000_003C, 4), Some(0));
    }

    #[test]
    fn ldr_str_roundtrip() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.set_x(1, 0x4000_0000);
        cpu.regs.set_x(0, 0xCAFE_0000_DEAD_BEEF);
        execute(&mut cpu, &mut bus, decode(0xF900_0020).unwrap()).unwrap();
        assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0xCAFE_0000_DEAD_BEEF));
        execute(&mut cpu, &mut bus, decode(0xF940_0022).unwrap()).unwrap();
        assert_eq!(cpu.regs.x(2), 0xCAFE_0000_DEAD_BEEF);
    }
}
