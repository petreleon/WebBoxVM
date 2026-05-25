//! AArch64 instruction decoding — pattern-based MVP decoder.

use super::Armv8Cpu;
use crate::bus::SystemBus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Add,
    Sub,
    Movz,
    Ldr,
    Str,
    B,
    Br,
    Nop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instr {
    pub op: Opcode,
    pub rd: u8,
    pub rn: u8,
    pub rm: u8,
    pub imm: u64,
    pub sf: bool,
}

/// Decode a raw 32-bit instruction.
pub fn decode(raw: u32) -> Option<Instr> {
    if raw == 0xD503_201F { return decode_nop(); }
    if ((raw >> 24) & 0x1F) == 0b11010 { return decode_dp_register(raw); }
    if ((raw >> 23) & 0x3F) == 0b100101 { return decode_movz(raw); }
    if ((raw >> 24) & 0xF8) == 0xF8 { return decode_ldst_unsigned(raw); }
    if ((raw >> 26) & 0x3F) == 0b000101 { return decode_b(raw); }
    if ((raw >> 24) & 0xFF) == 0xD6 { return decode_br(raw); }
    None
}

fn decode_nop() -> Option<Instr> {
    Some(Instr { op: Opcode::Nop, rd: 0, rn: 0, rm: 0, imm: 0, sf: true })
}

fn decode_dp_register(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let s = ((raw >> 29) & 1) != 0;
    let shift = ((raw >> 22) & 3) as u8;
    let n = ((raw >> 21) & 1) != 0;
    if s || shift != 0 || n { return None; }
    let rm = ((raw >> 16) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;
    let opcode = if op == 0 { Opcode::Add } else { Opcode::Sub };
    Some(Instr { op: opcode, rd, rn, rm, imm: 0, sf })
}

fn decode_movz(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    if ((raw >> 29) & 3) != 2 { return None; }
    let hw = ((raw >> 21) & 3) as u64;
    if hw > (if sf { 3 } else { 1 }) { return None; }
    let imm16 = ((raw >> 5) & 0xFFFF) as u64;
    let rd = (raw & 0x1F) as u8;
    Some(Instr { op: Opcode::Movz, rd, rn: 0, rm: 0, imm: imm16 << (hw * 16), sf })
}

fn decode_ldst_unsigned(raw: u32) -> Option<Instr> {
    let size = (raw >> 30) & 3;
    let _v = ((raw >> 29) & 1) != 0;
    let l = ((raw >> 22) & 1) != 0;
    if size != 3 { return None; }
    let imm12 = ((raw >> 10) & 0xFFF) as u64;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let op = if l { Opcode::Ldr } else { Opcode::Str };
    Some(Instr { op, rd: rt, rn, rm: 0, imm: imm12 << 3, sf: true })
}

fn decode_b(raw: u32) -> Option<Instr> {
    let imm26 = (raw & 0x3FF_FFFF) as i32;
    let offset = (imm26 << 6) >> 4;
    Some(Instr { op: Opcode::B, rd: 0, rn: 0, rm: 0, imm: offset as u64, sf: true })
}

fn decode_br(raw: u32) -> Option<Instr> {
    let rn = ((raw >> 5) & 0x1F) as u8;
    Some(Instr { op: Opcode::Br, rd: 0, rn, rm: 0, imm: 0, sf: true })
}

/// Execute a decoded instruction, mutating CPU and bus state.
pub fn execute(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    match instr.op {
        Opcode::Add => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) + read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Sub => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) - read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Movz => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::Ldr => {
            let addr = addr_with_offset(cpu, instr.rn, instr.imm)?;
            let size = if instr.sf { 8 } else { 4 };
            let val = bus.read(addr, size).ok_or("LDR bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Str => {
            let addr = addr_with_offset(cpu, instr.rn, instr.imm)?;
            let val = read_reg(cpu, instr.rd, instr.sf);
            let size = if instr.sf { 8 } else { 4 };
            bus.write(addr, size, val);
        }
        Opcode::B => {
            cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
            return Ok(());
        }
        Opcode::Br => {
            cpu.regs.pc = read_reg(cpu, instr.rn, true);
            return Ok(());
        }
        Opcode::Nop => {}
    }
    cpu.regs.pc += 4;
    Ok(())
}

fn read_reg(cpu: &Armv8Cpu, n: u8, sf: bool) -> u64 {
    assert!(n < 31, "register index must be 0-30");
    if sf { cpu.regs.x(n) } else { cpu.regs.w(n) as u64 }
}

fn write_reg(cpu: &mut Armv8Cpu, n: u8, val: u64, sf: bool) {
    assert!(n < 31, "register index must be 0-30");
    if sf { cpu.regs.set_x(n, val) } else { cpu.regs.set_w(n, val as u32) }
}

fn addr_with_offset(cpu: &Armv8Cpu, base: u8, offset: u64) -> Result<u64, &'static str> {
    if base >= 31 { return Err("SP not yet supported as base"); }
    Ok(cpu.regs.x(base).wrapping_add(offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_bus() -> (Armv8Cpu, SystemBus) {
        (Armv8Cpu::new(), SystemBus::new())
    }

    #[test]
    fn add_x0_x1_x2() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(1, 10);
        cpu.regs.set_x(2, 32);
        execute(&mut cpu, &mut bus, decode(0x9A02_0020).unwrap()
        ).unwrap();
        assert_eq!(cpu.regs.x(0), 42);
    }

    #[test]
    fn sub_x0_x1_x2() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(1, 50);
        cpu.regs.set_x(2, 8);
        execute(&mut cpu, &mut bus, decode(0xDA02_0020).unwrap()
        ).unwrap();
        assert_eq!(cpu.regs.x(0), 42);
    }

    #[test]
    fn movz_lsl_0() {
        let instr = decode(0xD282_4680).unwrap();
            assert_eq!(instr.op, Opcode::Movz);
        assert_eq!(instr.imm, 0x1234);
    }

    #[test]
    fn movz_lsl_16() {
        let instr = decode(0xD2A2_4680).unwrap();
        assert_eq!(instr.imm, 0x1234_0000);
    }

    #[test]
    fn ldr_str_roundtrip() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(1, 0x4000_0000);
        cpu.regs.set_x(0, 0xCAFE_0000_DEAD_BEEF);
        execute(&mut cpu, &mut bus, decode(0xF900_0020).unwrap()
        ).unwrap();
        assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0xCAFE_0000_DEAD_BEEF));
        execute(&mut cpu, &mut bus, decode(0xF940_0022).unwrap()
        ).unwrap();
        assert_eq!(cpu.regs.x(2), 0xCAFE_0000_DEAD_BEEF);
    }

    #[test]
    fn nop_advances_pc() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.pc = 0x4000_0000;
        execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()
        ).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0004);
    }

    #[test]
    fn branch_forward_4_bytes() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.pc = 0x4000_0000;
        execute(&mut cpu, &mut bus, decode(0x1400_0002).unwrap()
        ).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0008);
    }
}
