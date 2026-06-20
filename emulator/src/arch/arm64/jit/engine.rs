use super::block;
use super::emitter_a64::A64Compiler;
use crate::arch::arm64::{Armv8Cpu, decode, execute, mmu::translate, opcodes::Instr};
use crate::memory::PhysicalMemory;
use crate::platform::virt::SystemBus;
use std::collections::HashMap;

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
            hits: 0,
            misses: 0,
            native_hits: 0,
            steps: 0,
        }
    }

    pub fn run(
        &mut self,
        cpu: &mut Armv8Cpu,
        bus: &mut SystemBus,
        entry: u64,
        max_steps: usize,
    ) -> Result<usize, &'static str> {
        cpu.regs.pc = entry;
        let max = max_steps as u64;
        let compile_native_blocks = std::env::var_os("WEBBOXVM_ENABLE_JIT_COMPILE").is_some();

        let mut last_progress = 0u64;
        while self.steps < max {
            let milestone = self.steps / 5_000_000;
            if milestone > last_progress {
                self.report_progress();
                last_progress = milestone;
            }

            let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, cpu.regs.pc)
                .map_err(|_| "PC translation fault")?;
            if self.run_native_block(cpu, bus, pa) {
                continue;
            }

            let instr = self.cached_or_decode(pa, &bus.mem)?;
            self.steps += 1;
            execute(cpu, bus, instr).inspect_err(|e| {
                eprintln!("JIT EXEC ERROR: {} at PC={:#018x}", e, cpu.regs.pc);
            })?;

            if compile_native_blocks
                && self.steps > 1_000_000
                && self.steps.is_multiple_of(5_000_000)
            {
                let _ = self.try_compile_block(cpu, bus);
            }
        }

        self.report_done();
        Ok(max_steps)
    }

    fn report_progress(&self) {
        eprintln!(
            "JIT: {:.1}M steps, {} native, {} pages",
            self.steps as f64 / 1_000_000.0,
            self.native_hits,
            self.pages.len()
        );
    }

    fn report_done(&self) {
        eprintln!(
            "JIT DONE: {:.0}M steps, {} native blocks, {} pages",
            self.steps as f64 / 1_000_000.0,
            self.compiler.block_count(),
            self.pages.len()
        );
    }

    fn run_native_block(&mut self, cpu: &mut Armv8Cpu, bus: &mut SystemBus, pa: u64) -> bool {
        let Some(block) = self.compiler.get(pa) else {
            return false;
        };
        self.native_hits += 1;
        let count = block.guest_instr_count;
        unsafe {
            block.execute(cpu, bus);
        }
        cpu.regs.pc = block.exit_pc;
        self.steps += count as u64;
        true
    }

    fn cached_or_decode(&mut self, pa: u64, mem: &PhysicalMemory) -> Result<Instr, &'static str> {
        if let Some(instr) = self.get_cached(pa, mem) {
            self.hits += 1;
            return Ok(instr);
        }
        self.misses += 1;
        self.decode_and_get(pa, mem)
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
            let Some(raw) = mem.read(addr, 4) else {
                break;
            };
            page.push(decode(raw as u32).unwrap_or_else(Instr::nop));
        }
        if offset >= page.len() {
            return Err("offset beyond page end");
        }
        let result = page[offset];
        self.pages.insert(page_base, page);
        Ok(result)
    }

    fn try_compile_block(&mut self, cpu: &Armv8Cpu, bus: &SystemBus) -> Result<(), &'static str> {
        if let Ok(blk) = block::block_from_pc(cpu, bus)
            && let Err(e) = self.compiler.compile(&blk, cpu, bus)
        {
            eprintln!("JIT COMPILE FAIL: {} at PC={:#x}", e, blk.start_pc);
        }
        Ok(())
    }
}

impl Default for JitEngine {
    fn default() -> Self {
        Self::new()
    }
}
