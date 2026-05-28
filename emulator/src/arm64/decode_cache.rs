//! Instruction decode cache: pre-decodes pages by physical address.
//! Cached by PA to avoid MMU invalidation issues. 4KB pages, 1024 instrs/page.

use super::decode;
use super::opcodes::Instr;
use crate::memory::PhysicalMemory;
use std::collections::HashMap;

pub struct DecodeCache {
    pages: HashMap<u64, Vec<Instr>>, // PA page_base → 1024 decoded Instrs
    pub hits: u64,
    pub misses: u64,
}

impl DecodeCache {
    pub fn new() -> Self {
        Self { pages: HashMap::new(), hits: 0, misses: 0 }
    }

    /// Fetch and decode instruction at physical address `pa`.
    /// Returns the decoded Instr, decoding the entire 4KB page on miss.
    pub fn fetch(&mut self, mem: &PhysicalMemory, pa: u64) -> Option<Instr> {
        let page_base = pa & !0xFFFu64;
        let offset = ((pa & 0xFFF) / 4) as usize;

        if let Some(page) = self.pages.get(&page_base) {
            self.hits += 1;
            return page.get(offset).copied();
        }

        self.misses += 1;
        let mut instrs: Vec<Option<Instr>> = vec![None; 1024];
        for i in 0..1024u64 {
            let addr = page_base + i * 4;
            if let Some(raw) = mem.read(addr, 4) {
                instrs[i as usize] = decode(raw as u32);
            }
        }

        let result = instrs[offset];
        // Only cache if we got at least something at the requested offset
        if result.is_some() {
            // Store only the successfully decoded pages
            let page: Vec<Instr> = instrs.into_iter().filter_map(|o| o).collect();
            // Only cache if the page is fully populated (1024 entries)
            if page.len() == 1024 {
                self.pages.insert(page_base, page);
            }
        }
        result
    }
}
