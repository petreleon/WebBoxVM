//! ARM64 → x86_64 JIT compiler: block-based template translation.
//! Translates ARM64 basic blocks into x86_64 machine code, caches
//! compiled blocks, and executes them directly on the host CPU.

mod block;
mod emitter_x64;
mod code_cache;

use super::Armv8Cpu;
use crate::bus::SystemBus;
use crate::arm64::mmu::translate;
use block::{Block, block_from_pc};
use code_cache::CodeCache;

/// JIT execution engine.
pub struct JitEngine {
    code_cache: CodeCache,
}

impl JitEngine {
    pub fn new() -> Self {
        Self { code_cache: CodeCache::new() }
    }

    /// Run up to `max_steps` instructions using JIT-compiled blocks.
    pub fn run(
        &mut self,
        cpu: &mut Armv8Cpu,
        bus: &mut SystemBus,
        entry: u64,
        max_steps: usize,
    ) -> Result<usize, &'static str> {
        cpu.regs.pc = entry;
        let mut steps = 0;

        while steps < max_steps {
            // Translate PC to PA (required for JIT block lookup)
            let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, cpu.regs.pc)
                .map_err(|_| "JIT: PC translation fault")?;

            // Check code cache for compiled block at this PA
            if let Some(compiled) = self.code_cache.get(pa) {
                let instr_count = compiled.arm64_instr_count;
                // Execute compiled x86_64 code directly
                unsafe {
                    compiled.execute(cpu, bus);
                }
                steps += instr_count;
                continue;
            }

            // Not cached: discover block, compile, cache, execute
            let block = block_from_pc(cpu, bus)?;
            let arm64_count = block.instructions.len();
            self.code_cache.compile(&block, cpu, bus)?;
            let compiled = self.code_cache.get(pa).unwrap();
            unsafe {
                compiled.execute(cpu, bus);
            }
            steps += arm64_count;
        }

        Ok(steps)
    }
}
