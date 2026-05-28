//! Basic block discovery: decode ARM64 instructions until terminator.
//! Does NOT execute — purely static analysis.

use super::super::{Armv8Cpu, decode, opcodes::{Instr, Opcode}};
use crate::bus::SystemBus;
use crate::arm64::mmu::translate;

pub struct Block {
    pub start_pc: u64,
    pub start_pa: u64,
    pub instructions: Vec<(Instr, u32)>, // (decoded, raw)
}

/// Discover block starting at current PC. Does NOT execute.
pub fn block_from_pc(cpu: &Armv8Cpu, bus: &SystemBus) -> Result<Block, &'static str> {
    let start_pc = cpu.regs.pc;
    let start_pa = translate(&cpu.sys, &mut cpu.tlb.clone(), &bus.mem, start_pc)
        .map_err(|_| "block_from_pc: translation fault")?;

    let mut instructions = Vec::new();
    let mut pc = start_pc;
    let mut tlb = cpu.tlb.clone();

    loop {
        let pa = translate(&cpu.sys, &mut tlb, &bus.mem, pc)
            .map_err(|_| "block_from_pc: translation fault")?;
        let raw = bus.mem.read(pa, 4).ok_or("block_from_pc: memory fault")? as u32;
        let instr = decode(raw).ok_or("block_from_pc: decode fault")?;

        let is_terminator = matches!(
            instr.op,
            Opcode::B | Opcode::Br | Opcode::Blr | Opcode::Ret
                | Opcode::Bl | Opcode::BCond | Opcode::Cbz | Opcode::Cbnz
                | Opcode::Tbz | Opcode::Tbnz | Opcode::Svc | Opcode::Brk
                | Opcode::Eret
        );

        instructions.push((instr, raw));
        pc += 4;

        if is_terminator || instructions.len() >= 64 {
            break;
        }
    }

    Ok(Block { start_pc, start_pa, instructions })
}
