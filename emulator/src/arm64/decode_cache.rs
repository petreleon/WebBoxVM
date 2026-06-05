//! Instruction decode cache — pre-decodes instruction pages indexed by physical address.
//!
//! Each entry covers one 4 KiB page (up to 1024 instructions).  The cache is
//! keyed by physical address so MMU/ASID changes don't invalidate it.

use super::decode;
use super::opcodes::Instr;
use crate::constants::*;
use crate::memory::PhysicalMemory;

const DECODE_CACHE_LINES: usize = 16 * 1024;

#[derive(Debug, Clone)]
struct DecodedPage {
    raw_words: Vec<u32>,
    instrs: Vec<Instr>,
}

#[derive(Debug, Clone)]
struct CacheLine {
    page_base: u64,
    page: DecodedPage,
}

/// Per-core instruction decode cache.
///
/// A cached page stores both the decoded instructions and their original raw
/// words. On fetch, the requested word is compared with memory before reusing
/// the page so boot-time patching or self-modifying code gets re-decoded.
pub struct DecodeCache {
    pages: Vec<Option<CacheLine>>,
    pub hits: u64,
    pub misses: u64,
}

impl DecodeCache {
    pub fn new() -> Self {
        Self {
            pages: vec![None; DECODE_CACHE_LINES],
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
        let slot = cache_slot(page_base);

        if let Some(line) = &self.pages[slot]
            && line.page_base == page_base
            && line.page.raw_words.get(word_offset).copied() == Some(raw)
        {
            self.hits += 1;
            return line.page.instrs.get(word_offset).copied();
        }

        self.misses += 1;
        let page = decode_page(mem, page_base)?;
        let result = page.instrs.get(word_offset).copied();
        self.pages[slot] = Some(CacheLine { page_base, page });
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

fn cache_slot(page_base: u64) -> usize {
    ((page_base / PAGE_SIZE) as usize) & (DECODE_CACHE_LINES - 1)
}

#[cfg(test)]
mod tests;
