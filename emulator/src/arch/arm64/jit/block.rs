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
mod tests {
    use super::*;
    use crate::constants::*;

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

    #[test]
    fn fetch_cache_retranslates_after_page_boundary() {
        let mut cpu = Armv8Cpu::default();
        let mut bus = SystemBus::new();
        map_user_page_one(&mut cpu, &mut bus, RAM_BASE + 0x3000);
        cpu.regs.pc = 0x1ffc;
        bus.mem.write(RAM_BASE + 0x3ffc, 4, 0xd503_201f);
        bus.mem.write(RAM_BASE + 0x4000, 4, 0xd503_201f);

        let block = block_from_pc(&cpu, &bus).expect("first mapped instruction should compile");

        assert_eq!(block.instruction_pas, vec![RAM_BASE + 0x3ffc]);
        assert_eq!(block.instructions.len(), 1);
    }

    fn map_user_page_one(cpu: &mut Armv8Cpu, bus: &mut SystemBus, pa: u64) {
        let l1 = RAM_BASE;
        let l2 = RAM_BASE + PAGE_SIZE;
        let l3 = RAM_BASE + 2 * PAGE_SIZE;
        bus.mem.write(l1, 8, (l2 & DESC_ADDR_MASK) | DESC_TABLE);
        bus.mem.write(l2, 8, (l3 & DESC_ADDR_MASK) | DESC_TABLE);
        bus.mem
            .write(l3 + 8, 8, (pa & DESC_ADDR_MASK) | DESC_VALID | DESC_AF_BIT);
        cpu.sys.ttbr0_el1 = l1;
        cpu.sys.tcr_el1 = (25 << TCR_T1SZ_SHIFT) | 25;
        cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE;
    }
}
