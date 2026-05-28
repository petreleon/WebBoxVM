//! JIT engine: pre-decode cache + ARM64→ARM64 native compilation.
//! Verbatim ALU/move ops execute at native speed on Apple Silicon.

use crate::arm64::{Armv8Cpu, decode, execute, opcodes::{Instr, Opcode}, mmu::translate};
use crate::bus::SystemBus;
use crate::memory::PhysicalMemory;
use std::collections::HashMap;

mod block;
mod emitter_a64;
use emitter_a64::A64Compiler;

pub struct JitEngine {
    pages: HashMap<u64, Vec<Instr>>,
    compiler: A64Compiler,
    pub hits: u64,
    pub misses: u64,
    pub native_hits: u64,
    pub steps: u64,
}

impl JitEngine {
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
            compiler: A64Compiler::new(),
            hits: 0, misses: 0, native_hits: 0, steps: 0,
        }
    }

    pub fn run(
        &mut self, cpu: &mut Armv8Cpu, bus: &mut SystemBus,
        entry: u64, max_steps: usize,
    ) -> Result<usize, &'static str> {
        cpu.regs.pc = entry;

        for step in 0..max_steps {
            if step % 5_000_000 == 0 {
                eprintln!("JIT: {:.1}M steps, {} native, {} pages", step as f64 / 1_000_000.0, self.native_hits, self.pages.len());
            }

            let pc = cpu.regs.pc;
            let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, pc)
                .map_err(|_| "PC translation fault")?;

            // Try native compiled block
            if let Some(block) = self.compiler.get(pa) {
                self.native_hits += 1;
                let count = block.guest_instr_count;
                unsafe { block.execute(cpu, bus); }
                // Update guest PC past the executed block
                cpu.regs.pc = block.exit_pc;
                self.steps += count as u64;
                continue;
            }

            // Fallback: pre-decode cache + interpreter
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

        eprintln!("JIT DONE: {} steps, {} native blocks, {} pages",
            self.steps, self.compiler.block_count(), self.pages.len());
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
            } else { break };
            page.push(instr);
        }
        if offset >= page.len() { return Err("offset beyond page end"); }
        let result = page[offset];
        self.pages.insert(page_base, page);
        Ok(result)
    }

    fn try_compile_block(&mut self, cpu: &Armv8Cpu, bus: &SystemBus) -> Result<(), &'static str> {
        match block::block_from_pc(cpu, bus) {
            Ok(blk) => {
                if let Err(e) = self.compiler.compile(&blk, cpu, bus) {
                    eprintln!("JIT COMPILE FAIL: {} at PC={:#x}", e, blk.start_pc);
                }
            }
            Err(e) => {
                // Translation faults during block discovery are expected for unmapped pages
                // Just skip silently
            }
        }
        Ok(())
    }
}
