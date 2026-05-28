//! Multi-core ARM64 machine: orchestrates N CPU cores sharing a SystemBus.
//! Round-robin scheduling: each core executes a timeslice, then yields.

use crate::arm64::{Armv8Cpu, decode, execute, mmu::translate, opcodes::{Instr, Opcode}};
use crate::bus::SystemBus;
use std::collections::HashMap;

/// Pre-decoded instruction page cache (per physical address).
type DecodeCache = HashMap<u64, Vec<Instr>>;

/// Multi-core virtual machine.
pub struct Machine {
    pub cpus: Vec<Armv8Cpu>,
    pub bus: SystemBus,
    /// Pre-decoded instruction caches, one per core
    caches: Vec<DecodeCache>,
    /// Timeslice: instructions per core per round
    timeslice: usize,
    /// Index of the currently active core
    pub active_core: usize,
    /// Global step counter
    pub total_steps: u64,
}

impl Machine {
    /// Create a machine with `num_cores` CPUs sharing a single bus.
    pub fn new(num_cores: usize) -> Self {
        let cpus: Vec<_> = (0..num_cores)
            .map(|i| Armv8Cpu::with_core(i as u32))
            .collect();
        let caches = (0..num_cores).map(|_| HashMap::new()).collect();
        Self {
            cpus,
            bus: SystemBus::new(),
            caches,
            timeslice: 10_000, // 10K instructions per core per round
            active_core: 0,
            total_steps: 0,
        }
    }

    /// Run up to `max_total_steps` across all cores, round-robin.
    /// Each core gets ~timeslice steps before yielding to the next core.
    pub fn run(&mut self, max_total_steps: usize) -> usize {
        let start_steps = self.total_steps;
        let num_cores = self.cpus.len();
        // Per-instruction round-robin: one instruction per core per iteration
        let instr_per_core = 1;

        while (self.total_steps - start_steps) < max_total_steps as u64 {
            let core = self.active_core;
            let cpu = &mut self.cpus[core];
            let cache = &mut self.caches[core];

            for _ in 0..instr_per_core {
                if (self.total_steps - start_steps) >= max_total_steps as u64 {
                    return (self.total_steps - start_steps) as usize;
                }

                let pc = cpu.regs.pc;
                let pa = match translate(&cpu.sys, &mut cpu.tlb, &self.bus.mem, pc) {
                    Ok(pa) => pa,
                    Err(_) => break,
                };

                let instr = get_cached_or_fetch(cache, &self.bus.mem, pa);

                if let Some(instr) = instr {
                    // Handle MSR to GIC for multi-core interrupt routing
                    if instr.op == Opcode::Msr {
                        let sysreg_id = instr.imm as u16;
                        if sysreg_id == 0x4661 {
                            // ICC_EOIR1_EL1 — end of interrupt
                            cpu.sys.irq_pending = false;
                            cpu.sys.last_irq_id = 1023;
                            cpu.regs.pc += 4;
                            self.total_steps += 1;
                            continue;
                        }
                        if sysreg_id == 0x4660 {
                            // ICC_IAR1_EL1 — acknowledge interrupt
                            let val = if cpu.sys.irq_pending {
                                cpu.sys.irq_pending = false;
                                cpu.sys.last_irq_id as u64
                            } else {
                                0x3FF
                            };
                            crate::arm64::write_reg(cpu, instr.rd, val, true);
                            cpu.regs.pc += 4;
                            self.total_steps += 1;
                            continue;
                        }
                    }

                    if let Err(_) = execute(cpu, &mut self.bus, instr) {
                        break;
                    }
                } else {
                    break;
                }

                self.total_steps += 1;
            }

            // Round-robin: advance to next core
            self.active_core = (core + 1) % num_cores;
        }

        (self.total_steps - start_steps) as usize
    }

    /// Get a specific core's CPU reference.
    pub fn core(&self, n: usize) -> &Armv8Cpu {
        &self.cpus[n]
    }

    /// Get a mutable reference to a specific core.
    pub fn core_mut(&mut self, n: usize) -> &mut Armv8Cpu {
        &mut self.cpus[n]
    }
}

/// Get cached instruction or decode the entire 4KB page.
fn get_cached_or_fetch(cache: &mut DecodeCache, mem: &crate::memory::PhysicalMemory, pa: u64) -> Option<Instr> {
    let page_base = pa & !0xFFFu64;
    let offset = ((pa & 0xFFF) / 4) as usize;

    if let Some(page) = cache.get(&page_base) {
        return page.get(offset).copied();
    }

    // Decode entire page
    let mut page: Vec<Instr> = Vec::with_capacity(1024);
    for i in 0..1024u64 {
        let addr = page_base + i * 4;
        let raw = mem.read(addr, 4)? as u32;
        page.push(decode(raw).unwrap_or(Instr {
            op: Opcode::Nop, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0, size: 0
        }));
    }
    let result = page.get(offset).copied();
    cache.insert(page_base, page);
    result
}
