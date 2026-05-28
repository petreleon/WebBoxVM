//! JIT engine: pre-decoded instruction cache + interpreter execution.
//! JIT compilation is incrementally enabled as instructions are covered.

use crate::arm64::{Armv8Cpu, decode, execute, opcodes::{Instr, Opcode}, mmu::translate};
use crate::bus::SystemBus;
use crate::memory::PhysicalMemory;
use std::collections::HashMap;

pub struct JitEngine {
    /// Pre-decoded instruction pages (PA → 1024 Instrs)
    pages: HashMap<u64, Vec<Instr>>,
    pub hits: u64,
    pub misses: u64,
    pub steps: u64,
}

impl JitEngine {
    pub fn new() -> Self {
        Self { pages: HashMap::new(), hits: 0, misses: 0, steps: 0 }
    }

    /// Run up to max_steps using cached pre-decoded instructions.
    pub fn run(
        &mut self, cpu: &mut Armv8Cpu, bus: &mut SystemBus,
        entry: u64, max_steps: usize,
    ) -> Result<usize, &'static str> {
        cpu.regs.pc = entry;

        for step in 0..max_steps {
            if step % 5_000_000 == 0 {
                eprintln!("JIT PROGRESS: {:.1}M steps", step as f64 / 1_000_000.0);
            }
            let pc = cpu.regs.pc;
            let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, pc)
                .map_err(|_| "PC translation fault")?;

            let instr = match self.get_cached(pa, &bus.mem) {
                Some(i) => { self.hits += 1; i }
                None => { self.misses += 1; self.decode_and_get(pa, &bus.mem)? }
            };

            self.steps += 1;
            execute(cpu, bus, instr).map_err(|e| {
                eprintln!("JIT EXEC ERROR: {} at PC={:#018x}", e, cpu.regs.pc);
                e
            })?;
        }

        let total = self.hits + self.misses;
        eprintln!("JIT: {} steps, {}/{} hits/misses ({:.1}% hit), {} pages",
            self.steps, self.hits, self.misses,
            if total > 0 { (self.hits as f64 / total as f64) * 100.0 } else { 0.0 },
            self.pages.len());

        Ok(max_steps)
    }

    fn get_cached(&self, pa: u64, _mem: &PhysicalMemory) -> Option<Instr> {
        let page_base = pa & !0xFFFu64;
        let offset = ((pa & 0xFFF) / 4) as usize;
        self.pages.get(&page_base)?.get(offset).copied()
    }

    fn decode_and_get(&mut self, pa: u64, mem: &PhysicalMemory) -> Result<Instr, &'static str> {
        let page_base = pa & !0xFFFu64;
        let offset = ((pa & 0xFFF) / 4) as usize;

        let mut page: Vec<Instr> = Vec::with_capacity(1024);
        for i in 0..1024u64 {
            let addr = page_base + i * 4;
            let instr = if let Some(raw) = mem.read(addr, 4) {
                decode(raw as u32).unwrap_or(Instr {
                    op: Opcode::Nop, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0, size: 0
                })
            } else {
                break; // end of mapped memory — stop here
            };
            page.push(instr);
        }

        if offset >= page.len() {
            return Err("offset beyond page end");
        }
        let result = page[offset];
        self.pages.insert(page_base, page);
        Ok(result)
    }
}
