//! Basic block discovery: decode ARM64 instructions until terminator.
//! Does NOT execute — purely static analysis.
//! Handles translation faults gracefully (partial blocks on fault).

use super::super::{
    Armv8Cpu, decode,
    opcodes::{Instr, Opcode},
};
use crate::arm64::mmu::translate;
use crate::bus::SystemBus;

pub struct Block {
    pub start_pc: u64,
    pub start_pa: u64,
    pub instruction_pas: Vec<u64>,
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
    let mut instruction_pas = Vec::new();
    let mut pc = start_pc;
    let mut tlb = cpu.tlb.clone();
    loop {
        if instructions.len() >= 64 {
            break;
        }
        // Translate PC → PA. On fault, end the block gracefully.
        let pa = match translate(&cpu.sys, &mut tlb, &bus.mem, pc) {
            Ok(pa) => pa,
            Err(_) => {
                if instructions.is_empty() {
                    return Err("block instruction translation fault");
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
                // Undecodable — probably data/BSS, end block
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
                | Opcode::Brk
                | Opcode::Eret
        );

        instructions.push((instr, raw));
        instruction_pas.push(pa);
        pc += 4;

        if is_terminator || instructions.len() >= 64 {
            break;
        }
    }

    if instructions.is_empty() {
        return Err("empty block");
    }

    Ok(Block {
        start_pc,
        start_pa,
        instruction_pas,
        instructions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::RAM_BASE;

    #[test]
    fn rejects_undecodable_instruction_at_start_pc() {
        let mut cpu = Armv8Cpu::default();
        let mut bus = SystemBus::new();
        cpu.regs.pc = RAM_BASE;
        bus.mem.write(RAM_BASE, 4, 0xffff_ffff);
        bus.mem.write(RAM_BASE + 4, 4, 0xd503_201f);

        assert_eq!(
            block_from_pc(&cpu, &bus).err(),
            Some("block starts with undecodable instruction")
        );
    }

    #[test]
    fn rejects_unreadable_instruction_at_start_pc() {
        let mut cpu = Armv8Cpu::default();
        let bus = SystemBus::new();
        cpu.regs.pc = 0x9000_0000;

        assert_eq!(
            block_from_pc(&cpu, &bus).err(),
            Some("block instruction read fault")
        );
    }
}
