//! Basic block discovery: decode ARM64 instructions until terminator.
//! Does NOT execute — purely static analysis.
//! Handles translation faults gracefully (partial blocks on fault).

use super::super::{
    Armv8Cpu, decode,
    opcodes::{Instr, Opcode},
};
use crate::arch::arm64::mmu::translate_read_only;
use crate::arch::arm64::system_regs::SystemRegisters;
use crate::constants::{PAGE_OFFSET_MASK, PAGE_SHIFT};
use crate::memory::PhysicalMemory;
use crate::platform::virt::SystemBus;

pub struct Block {
    pub start_pc: u64,
    pub start_pa: u64,
    pub instruction_pas: Vec<u64>,
    pub instructions: Vec<(Instr, u32)>, // (decoded, raw)
}

pub(crate) const MAX_BLOCK_INSTRUCTIONS: usize = 64;

/// Discover block at current PC. Returns partial block on fault.
pub fn block_from_pc(cpu: &Armv8Cpu, bus: &SystemBus) -> Result<Block, &'static str> {
    let start_pc = cpu.regs.pc;
    let mut instructions = Vec::new();
    let mut instruction_pas = Vec::new();
    let mut pc = start_pc;
    let mut translated_page = None;
    loop {
        if instructions.len() >= MAX_BLOCK_INSTRUCTIONS {
            break;
        }
        let pa = match translate_fetch_pc(&cpu.sys, &cpu.tlb, &bus.mem, pc, &mut translated_page) {
            Ok(pa) => pa,
            Err(_) => {
                if instructions.is_empty() {
                    return Err("block start translation fault");
                } else {
                    break;
                }
            }
        };

        let raw = match bus.mem.read(pa, 4) {
            Some(value) => value as u32,
            None if instructions.is_empty() => return Err("block instruction read fault"),
            None => break,
        };

        let instr = match decode(raw) {
            Some(i) => i,
            None => {
                if instructions.is_empty() {
                    return Err("block starts with undecodable instruction");
                }
                break;
            }
        };

        let is_terminator = matches!(
            instr.op,
            Opcode::B
                | Opcode::Br
                | Opcode::Blr
                | Opcode::Ret
                | Opcode::Bl
                | Opcode::BCond
                | Opcode::Cbz
                | Opcode::Cbnz
                | Opcode::Tbz
                | Opcode::Tbnz
                | Opcode::Svc
                | Opcode::Hvc
                | Opcode::Brk
                | Opcode::Eret
        );

        instructions.push((instr, raw));
        instruction_pas.push(pa);
        pc += 4;

        if is_terminator || instructions.len() >= MAX_BLOCK_INSTRUCTIONS {
            break;
        }
    }

    if instructions.is_empty() {
        return Err("empty block");
    }

    let start_pa = instruction_pas[0];
    Ok(Block {
        start_pc,
        start_pa,
        instruction_pas,
        instructions,
    })
}

fn translate_fetch_pc(
    sys: &SystemRegisters,
    tlb: &crate::arch::arm64::Tlb,
    mem: &PhysicalMemory,
    pc: u64,
    translated_page: &mut Option<(u64, u64)>,
) -> Result<u64, ()> {
    let va_page = pc >> PAGE_SHIFT;
    if let Some((cached_va_page, cached_pa_page)) = *translated_page {
        if cached_va_page == va_page {
            return Ok((cached_pa_page << PAGE_SHIFT) | (pc & PAGE_OFFSET_MASK));
        }
    }
    let pa = translate_read_only(sys, Some(tlb), mem, pc).map_err(|_| ())?;
    if va_page != 0 {
        *translated_page = Some((va_page, pa >> PAGE_SHIFT));
    }
    Ok(pa)
}

#[cfg(test)]
mod tests;
