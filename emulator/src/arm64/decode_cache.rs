//! Instruction decode cache — pre-decodes instruction pages indexed by physical address.
//!
//! Each entry covers one 4 KiB page (up to 1024 instructions).  The cache is
//! keyed by physical address so MMU/ASID changes don't invalidate it.

use super::decode;
use super::opcodes::Instr;
use crate::memory::PhysicalMemory;
use crate::constants::*;
use std::collections::HashMap;

pub struct DecodeCache {
    pages: HashMap<u64, Vec<Instr>>,  // page_phys_addr → pre-decoded instruction list
    pub hits: u64,
    pub misses: u64,
}

impl DecodeCache {
    pub fn new() -> Self {
        Self { pages: HashMap::new(), hits: 0, misses: 0 }
    }

    /// Fetch and decode the instruction at physical address `pa`.
    /// On cache miss, the entire 4 KiB page is decoded and cached.
    pub fn fetch(&mut self, mem: &PhysicalMemory, pa: u64) -> Option<Instr> {
        let page_base = pa & !PAGE_OFFSET_MASK;
        let word_offset = ((pa & PAGE_OFFSET_MASK) / INSTRUCTION_SIZE) as usize;

        if let Some(page) = self.pages.get(&page_base) {
            self.hits += 1;
            return page.get(word_offset).copied();
        }

        self.misses += 1;
        let mut instrs: Vec<Option<Instr>> = vec![None; INSTRUCTIONS_PER_PAGE];
        for i in 0..INSTRUCTIONS_PER_PAGE as u64 {
            let addr = page_base + i * INSTRUCTION_SIZE;
            if let Some(raw) = mem.read(addr, 4) {
                instrs[i as usize] = decode(raw as u32);
            }
        }

        let result = instrs[word_offset];
        if result.is_some() {
            let page: Vec<Instr> = instrs.into_iter().filter_map(|o| o).collect();
            if page.len() == INSTRUCTIONS_PER_PAGE {
                self.pages.insert(page_base, page);
            }
        }
        result
    }
}
