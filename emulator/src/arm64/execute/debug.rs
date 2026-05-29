//! Debug dump helpers — used by the BRK handler.

use crate::arm64::helpers::read_reg;
use crate::arm64::mmu::translate;
use crate::arm64::{Armv8Cpu, decode, Tlb};
use crate::bus::SystemBus;
use crate::constants::*;
use super::{Instr};
use super::branch::branch_target;

pub(super) fn dump_instructions(label: &str, addr: u64, cpu: &Armv8Cpu, bus: &SystemBus) {
    let mut scratch_tlb = Tlb::new();
    eprintln!("Instructions around {} ({:#018x}):", label, addr);
    for offset in (-32..=32).step_by(4) {
        let target = branch_target(addr, offset as u64);
        if let Ok(pa) = translate(&cpu.sys, &mut scratch_tlb, &bus.mem, target) {
            if let Some(val) = bus.mem.read(pa, 4) {
                let decoded = decode(val as u32);
                eprintln!("  {:#018x}: {:08x} {:?}", target, val, decoded.map(|d| d.op));
            }
        }
    }
}

pub(super) fn dump_string_pointers(cpu: &Armv8Cpu, bus: &SystemBus) {
    for (i, &reg_val) in [cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2), cpu.regs.x(3), cpu.regs.x(4)].iter().enumerate() {
        if reg_val == 0 { continue; }
        let mut scratch_tlb = Tlb::new();
        if let Some(s) = try_read_string_at(bus, &mut scratch_tlb, &cpu.sys, reg_val) {
            if !s.is_empty() && s.len() > 2 {
                eprintln!("  maybe @X{}: \"{}\"", i, s);
            }
        }
    }
}

fn try_read_string_at(bus: &SystemBus, tlb: &mut Tlb, sys: &crate::arm64::SystemRegisters, addr: u64) -> Option<String> {
    let mut s = String::new();
    for off in 0..128u64 {
        match translate(sys, tlb, &bus.mem, addr + off) {
            Ok(pa) => {
                if let Some(val) = bus.mem.read(pa, 1) {
                    let byte = val as u8;
                    if byte == 0 { break; }
                    if byte.is_ascii_graphic() || byte == b' ' { s.push(byte as char); } else { break; }
                } else { break; }
            }
            Err(_) => break,
        }
    }
    if s.is_empty() { None } else { Some(s) }
}

pub(super) fn dump_stack(cpu: &Armv8Cpu, bus: &SystemBus) {
    let sp = cpu.regs.sp;
    eprintln!("Stack around SP={:#018x}:", sp);
    let mut scratch_tlb = Tlb::new();
    for offset in (-64..=64).step_by(8) {
        let addr = branch_target(sp, offset as u64);
        if let Ok(pa) = translate(&cpu.sys, &mut scratch_tlb, &bus.mem, addr) {
            if let Some(val) = bus.mem.read(pa, 8) {
                eprintln!("  {:#018x}: {:016x}", addr, val);
            }
        }
    }
}
