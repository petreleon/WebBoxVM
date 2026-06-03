//! Instruction decode cache — pre-decodes instruction pages indexed by physical address.
//!
//! Each entry covers one 4 KiB page (up to 1024 instructions).  The cache is
//! keyed by physical address so MMU/ASID changes don't invalidate it.

use super::decode;
use super::opcodes::Instr;
use crate::constants::*;
use crate::memory::PhysicalMemory;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct DecodedPage {
    raw_words: Vec<u32>,
    instrs: Vec<Instr>,
}

/// Per-core instruction decode cache.
///
/// A cached page stores both the decoded instructions and their original raw
/// words. On fetch, the requested word is compared with memory before reusing
/// the page so boot-time patching or self-modifying code gets re-decoded.
pub struct DecodeCache {
    pages: HashMap<u64, DecodedPage>,
    pub hits: u64,
    pub misses: u64,
}

impl DecodeCache {
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Fetch and decode the instruction at physical address `pa`.
    /// On cache miss, the entire 4 KiB page is decoded and cached. Undecoded
    /// words are represented as NOPs to match the interpreter's tolerant fetch
    /// behavior.
    pub fn fetch(&mut self, mem: &PhysicalMemory, pa: u64) -> Option<Instr> {
        let page_base = pa & !PAGE_OFFSET_MASK;
        let word_offset = ((pa & PAGE_OFFSET_MASK) / INSTRUCTION_SIZE) as usize;
        let raw = read_raw_word(mem, pa)?;

        if let Some(page) = self.pages.get(&page_base)
            && page.raw_words.get(word_offset).copied() == Some(raw)
        {
            self.hits += 1;
            return page.instrs.get(word_offset).copied();
        }

        self.misses += 1;
        let page = decode_page(mem, page_base)?;
        let result = page.instrs.get(word_offset).copied();
        self.pages.insert(page_base, page);
        result
    }
}

impl Default for DecodeCache {
    fn default() -> Self {
        Self::new()
    }
}

fn decode_page(mem: &PhysicalMemory, page_base: u64) -> Option<DecodedPage> {
    let mut raw_words = Vec::with_capacity(INSTRUCTIONS_PER_PAGE);
    let mut instrs = Vec::with_capacity(INSTRUCTIONS_PER_PAGE);

    for i in 0..INSTRUCTIONS_PER_PAGE as u64 {
        let raw = read_raw_word(mem, page_base + i * INSTRUCTION_SIZE)?;
        raw_words.push(raw);
        instrs.push(decode(raw).unwrap_or_else(Instr::nop));
    }

    Some(DecodedPage { raw_words, instrs })
}

fn read_raw_word(mem: &PhysicalMemory, pa: u64) -> Option<u32> {
    mem.read(pa, 4).map(|raw| raw as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arm64::Opcode;
    use crate::constants::RAM_BASE;

    #[test]
    fn cached_fetch_reuses_unchanged_page() {
        let mut mem = PhysicalMemory::new();
        let mut cache = DecodeCache::new();

        mem.write(RAM_BASE, 4, 0xd503_201f).unwrap();

        assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::Nop);
        assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::Nop);
        assert_eq!(cache.misses, 1);
        assert_eq!(cache.hits, 1);
    }

    #[test]
    fn cached_fetch_redecodes_changed_word() {
        let mut mem = PhysicalMemory::new();
        let mut cache = DecodeCache::new();

        mem.write(RAM_BASE, 4, 0xd503_201f).unwrap();
        assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::Nop);

        mem.write(RAM_BASE, 4, 0x1400_0000).unwrap();

        assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::B);
        assert_eq!(cache.misses, 2);
    }

    #[test]
    fn undecoded_words_are_tolerated_as_nops() {
        let mut mem = PhysicalMemory::new();
        let mut cache = DecodeCache::new();

        mem.write(RAM_BASE, 4, 0xffff_ffff).unwrap();

        assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::Nop);
    }
}
