//! Basic block discovery: scan ARM64 instructions until a terminator.

use super::super::{Armv8Cpu, decode, execute, opcodes::{Instr, Opcode}};
use crate::bus::SystemBus;
use crate::arm64::mmu::translate;

/// A basic block: sequence of non-branch instructions terminated by a branch.
pub struct Block {
    pub start_pc: u64,
    pub start_pa: u64,
    pub instructions: Vec<Instr>,
    pub successors: Vec<u64>, // possible next PC values (fallthrough + branch targets)
}

/// Discover a basic block starting at the current PC.
/// Executes instructions via the interpreter during discovery
/// to determine branch targets and side effects.
pub fn block_from_pc(cpu: &mut Armv8Cpu, bus: &mut SystemBus) -> Result<Block, &'static str> {
    let start_pc = cpu.regs.pc;
    let start_pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, start_pc)
        .map_err(|_| "block_from_pc: translation fault")?;

    let mut instructions = Vec::new();
    let mut pc = start_pc;

    loop {
        let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, pc)
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

        instructions.push(instr);
        pc += 4;

        if is_terminator || instructions.len() >= 64 {
            break;
        }
    }

    // Determine successors
    let mut successors = Vec::new();
    let last = instructions.last().unwrap();
    match last.op {
        // Direct branch: target is known
        Opcode::B | Opcode::Bl => {
            let target = (start_pc as i64 + last.imm as i64) as u64;
            successors.push(target);
        }
        // Conditional branch: fallthrough + target
        Opcode::BCond | Opcode::Cbz | Opcode::Cbnz | Opcode::Tbz | Opcode::Tbnz => {
            successors.push(start_pc + (instructions.len() as u64) * 4); // fallthrough
            let target = (start_pc as i64 + last.imm as i64) as u64;
            successors.push(target);
        }
        // Indirect branch: target unknown at compile time
        Opcode::Br | Opcode::Blr | Opcode::Ret => {
            successors.push(0); // unknown
        }
        // Exception: no normal successor
        Opcode::Svc | Opcode::Brk | Opcode::Eret => {
            // handled by interpreter
        }
        // Fallthrough (block reached size limit)
        _ => {
            successors.push(start_pc + (instructions.len() as u64) * 4);
        }
    }

    // Execute the block via interpreter (collects side effects, updates PC)
    for instr in &instructions {
        execute(cpu, bus, *instr)?;
    }

    // If the last instruction was a branch that wasn't taken (B.cond with false condition),
    // execute() may have fallen through. Record the actual PC after execution.
    // The block discovery is done; the execution result determines where we go next.

    Ok(Block {
        start_pc,
        start_pa,
        instructions,
        successors,
    })
}
