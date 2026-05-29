//! Multi-core ARM64 machine — orchestrates N CPU cores sharing one SystemBus.
//!
//! Each core runs one instruction at a time in round-robin fashion,
//! with a per-core decode cache to avoid re-decoding the same page.

use crate::arm64::{Armv8Cpu, decode, execute, translate, Instr, Opcode};
use crate::bus::SystemBus;
use crate::constants::*;
use std::collections::HashMap;

/// Pre-decoded instruction cache (per physical page address).
type DecodeCache = HashMap<u64, Vec<Instr>>;

/// Multi-core virtual machine with shared memory bus.
pub struct Machine {
    pub cpus: Vec<Armv8Cpu>,
    pub bus: SystemBus,
    caches: Vec<DecodeCache>,   // one decode cache per core
    pub active_core: usize,
    pub total_steps: u64,
    pub fetch_faults: u64,
    pub exec_faults: u64,
    /// Ring buffer for recent instruction trace: (PC, opcode)
    instr_trace: Vec<(u64, Opcode)>,
    trace_idx: usize,
}

impl Machine {
    /// Create a machine with `num_cores` CPUs sharing a single system bus.
    pub fn new(num_cores: usize) -> Self {
        let cpus: Vec<_> = (0..num_cores)
            .map(|i| Armv8Cpu::with_core(i as u32))
            .collect();
        let caches = (0..num_cores).map(|_| HashMap::new()).collect();
        Self {
            cpus,
            bus: SystemBus::new(),
            caches,
            active_core: 0,
            total_steps: 0,
            fetch_faults: 0,
            exec_faults: 0,
            instr_trace: vec![(0, Opcode::Nop); 256],
            trace_idx: 0,
        }
    }

    /// Run up to `max_total_steps` across all cores using round-robin.
    /// Each core executes one instruction per turn.
    pub fn run(&mut self, max_total_steps: usize) -> usize {
        let start_steps = self.total_steps;
        let num_cores = self.cpus.len();
        let mut report_interval = 1_000_000u64;

        while (self.total_steps - start_steps) < max_total_steps as u64 {
            let core = self.active_core;
            let cpu = &mut self.cpus[core];
            let cache = &mut self.caches[core];

            if (self.total_steps - start_steps) >= max_total_steps as u64 {
                break;
            }

            // Periodic diagnostic report
            if (self.total_steps - start_steps) > 0 && (self.total_steps - start_steps) % report_interval == 0 {
                let pc = cpu.regs.pc;
                eprintln!("DIAG {:>9}M steps | fetch_faults={:>7} exec_faults={:>7} | PC=0x{:016x}",
                    (self.total_steps - start_steps) / 1_000_000,
                    self.fetch_faults, self.exec_faults, pc);
                if (self.total_steps - start_steps) >= 10_000_000 {
                    report_interval = 100_000_000;
                }
            }

            let pc = cpu.regs.pc;

            let pa = match translate(&cpu.sys, &mut cpu.tlb, &self.bus.mem, pc) {
                Ok(pa) => pa,
                Err(_) => {
                    self.fetch_faults += 1;
                    cpu.regs.pc += INSTRUCTION_SIZE;
                    self.total_steps += 1;
                    self.active_core = (core + 1) % num_cores;
                    continue;
                }
            };

            let instr = get_cached_or_fetch(cache, &self.bus.mem, pa);

            if let Some(instr) = instr {
                // Record in instruction trace ring buffer
                self.instr_trace[self.trace_idx] = (pc, instr.op);
                self.trace_idx = (self.trace_idx + 1) & 0xFF;

                // Intercept GIC system register accesses for interrupt routing
                if instr.op == Opcode::Msr {
                    let sysreg_id = instr.imm as u16;
                    match sysreg_id {
                        SYSREG_ICC_EOIR1_EL1 => {
                            cpu.sys.irq_pending = false;
                            cpu.sys.last_irq_id = GIC_SPURIOUS_INTERRUPT as u32;
                            cpu.regs.pc += INSTRUCTION_SIZE;
                            self.total_steps += 1;
                            self.active_core = (core + 1) % num_cores;
                            continue;
                        }
                        SYSREG_ICC_IAR1_EL1 => {
                            let val = if cpu.sys.irq_pending {
                                cpu.sys.irq_pending = false;
                                cpu.sys.last_irq_id as u64
                            } else {
                                GIC_SPURIOUS_INTERRUPT
                            };
                            crate::arm64::write_reg(cpu, instr.rd, val, true);
                            cpu.regs.pc += INSTRUCTION_SIZE;
                            self.total_steps += 1;
                            self.active_core = (core + 1) % num_cores;
                            continue;
                        }
                        _ => {}
                    }
                }

                if let Err(_) = execute(cpu, &mut self.bus, instr) {
                    self.exec_faults += 1;
                    cpu.regs.pc += INSTRUCTION_SIZE;
                    self.total_steps += 1;
                    self.active_core = (core + 1) % num_cores;
                    continue;
                }
            } else {
                // Decode failed — skip the bad instruction
                cpu.regs.pc += INSTRUCTION_SIZE;
            }

            self.total_steps += 1;
            self.active_core = (core + 1) % num_cores;
        }

        (self.total_steps - start_steps) as usize
    }

    pub fn core(&self, n: usize) -> &Armv8Cpu { &self.cpus[n] }
    pub fn core_mut(&mut self, n: usize) -> &mut Armv8Cpu { &mut self.cpus[n] }
}

/// Fetch one instruction, using the decode cache to avoid re-decoding.
/// If the page isn't cached yet, decode the entire 4 KiB page.
fn get_cached_or_fetch(
    cache: &mut DecodeCache,
    mem: &crate::memory::PhysicalMemory,
    pa: u64,
) -> Option<Instr> {
    let page_base = pa & !PAGE_OFFSET_MASK;
    let word_offset = ((pa & PAGE_OFFSET_MASK) / INSTRUCTION_SIZE) as usize;

    if let Some(page) = cache.get(&page_base) {
        return page.get(word_offset).copied();
    }

    // Decode the entire page (1024 instructions max)
    let mut page: Vec<Instr> = Vec::with_capacity(INSTRUCTIONS_PER_PAGE);
    for i in 0..INSTRUCTIONS_PER_PAGE as u64 {
        let addr = page_base + i * INSTRUCTION_SIZE;
        let raw = mem.read(addr, 4)? as u32;
        page.push(decode(raw).unwrap_or(Instr {
            op: Opcode::Nop, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0, size: 0
        }));
    }
    let result = page.get(word_offset).copied();
    cache.insert(page_base, page);
    result
}
