//! Instruction decode cache — pre-decodes instruction pages indexed by physical address.
//!
//! Each entry covers one 4 KiB page (up to 1024 instructions).  The cache is
//! keyed by physical address so MMU/ASID changes don't invalidate it.

use super::decode;
use super::opcodes::Instr;
use crate::constants::*;
use crate::memory::PhysicalMemory;

const DECODE_CACHE_LINES: usize = 16 * 1024;
type DecodedInstrs = Box<[Instr; INSTRUCTIONS_PER_PAGE]>;

#[derive(Debug, Clone)]
struct DecodedPage {
    generation: u64,
    instrs: DecodedInstrs,
}

#[derive(Debug, Clone)]
struct CacheLine {
    page_base: u64,
    page: DecodedPage,
}

/// Per-core instruction decode cache.
///
/// A cached page stores decoded instructions for one physical page. On fetch,
/// the page generation is compared with memory before reusing the page so
/// boot-time patching or self-modifying code gets re-decoded.
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
        let generation = mem.page_generation(page_base)?;
        let slot = cache_slot(page_base);

        if let Some(line) = &self.pages[slot]
            && line.page_base == page_base
            && line.page.generation == generation
        {
            self.hits += 1;
            return Some(line.page.instrs[word_offset]);
        }

        self.misses += 1;
        let page = decode_page(mem, page_base, generation)?;
        let result = page.instrs[word_offset];
        self.pages[slot] = Some(CacheLine { page_base, page });
        Some(result)
    }
}

impl Default for DecodeCache {
    fn default() -> Self {
        Self::new()
    }
}

fn decode_page(mem: &PhysicalMemory, page_base: u64, generation: u64) -> Option<DecodedPage> {
    let mut page_bytes = [0u8; PAGE_SIZE as usize];
    mem.read_bytes(page_base, &mut page_bytes)?;
    let instrs = Box::new(std::array::from_fn(|i| decode_page_word(&page_bytes, i)));

    Some(DecodedPage { generation, instrs })
}

fn decode_page_word(page_bytes: &[u8; PAGE_SIZE as usize], index: usize) -> Instr {
    let offset = index * INSTRUCTION_SIZE as usize;
    let raw = u32::from_le_bytes([
        page_bytes[offset],
        page_bytes[offset + 1],
        page_bytes[offset + 2],
        page_bytes[offset + 3],
    ]);
    decode(raw).unwrap_or_else(Instr::nop)
}

fn cache_slot(page_base: u64) -> usize {
    ((page_base / PAGE_SIZE) as usize) & (DECODE_CACHE_LINES - 1)
}

#[cfg(test)]
mod tests;
