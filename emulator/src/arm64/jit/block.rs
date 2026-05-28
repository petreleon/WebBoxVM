//! Basic block discovery: decode ARM64 instructions until terminator.
//! Does NOT execute — purely static analysis.
//! Handles translation faults gracefully (partial blocks on fault).

use super::super::{Armv8Cpu, decode, opcodes::{Instr, Opcode}};
use crate::bus::SystemBus;
use crate::arm64::mmu::translate;

pub struct Block {
    pub start_pc: u64,
    pub start_pa: u64,
    pub instructions: Vec<(Instr, u32)>, // (decoded, raw)
}

/// Discover block at current PC. Returns partial block on fault.
pub fn block_from_pc(cpu: &Armv8Cpu, bus: &SystemBus) -> Result<Block, &'static str> {
    let start_pc = cpu.regs.pc;
    let start_pa = match translate(&cpu.sys, &mut cpu.tlb.clone(), &bus.mem, start_pc) {
        Ok(pa) => pa,
        Err(_) => return Err("block start translation fault"),
    };

    let mut instructions = Vec::new();
    let mut pc = start_pc;
    let mut tlb = cpu.tlb.clone();
    let mut consecutive_faults = 0u32;
    let max_iterations = 256; // safety limit

    loop {
        if instructions.len() >= 64 || instructions.len() >= max_iterations {
            break;
        }
        // Translate PC → PA. On fault, end the block gracefully.
        let pa = match translate(&cpu.sys, &mut tlb, &bus.mem, pc) {
            Ok(pa) => {
                consecutive_faults = 0;
                pa
            }
            Err(_) => {
                consecutive_faults += 1;
                if consecutive_faults > 3 { break; }
                pc += 4;
                continue;
            }
        };
        consecutive_faults = 0;

        let raw = match bus.mem.read(pa, 4) {
            Some(v) => v as u32,
            None => break, // unmapped memory — end block
        };

        let instr = match decode(raw) {
            Some(i) => i,
            None => {
                // Undecodable — probably data/BSS, end block
                if instructions.is_empty() {
                    // At least one instruction needed
                    pc += 4;
                    continue;
                }
                break;
            }
        };

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

    if instructions.is_empty() {
        return Err("empty block");
    }

    Ok(Block { start_pc, start_pa, instructions })
}
