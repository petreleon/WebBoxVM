//! Instruction decode cache — caches instruction pages indexed by physical address.
//!
//! Each entry covers one 4 KiB page (up to 1024 instructions), but individual
//! words are decoded lazily. The cache is keyed by physical address so MMU/ASID
//! changes don't invalidate it.

use super::decode;
use super::opcodes::Instr;
use crate::constants::*;
use crate::memory::PhysicalMemory;

const DECODE_CACHE_LINES: usize = 16 * 1024;
type DecodedInstrs = Box<[Option<Instr>; INSTRUCTIONS_PER_PAGE]>;

#[derive(Debug, Clone)]
struct DecodedPage {
    generation: u64,
    instrs: DecodedInstrs,
}

impl DecodedPage {
    fn new(generation: u64) -> Self {
        Self {
            generation,
            instrs: Box::new([None; INSTRUCTIONS_PER_PAGE]),
        }
    }
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
    pub word_decodes: u64,
}

impl DecodeCache {
    pub fn new() -> Self {
        Self {
            pages: vec![None; DECODE_CACHE_LINES],
            hits: 0,
            misses: 0,
            word_decodes: 0,
        }
    }

    /// Fetch and decode the instruction at physical address `pa`.
    /// On cache miss, the 4 KiB page generation is cached and the requested
    /// word is decoded. Other words are decoded on demand. Undecoded words are
    /// represented as NOPs to match the interpreter's tolerant fetch behavior.
    pub fn fetch(&mut self, mem: &PhysicalMemory, pa: u64) -> Option<Instr> {
        let page_base = pa & !PAGE_OFFSET_MASK;
        let word_offset = ((pa & PAGE_OFFSET_MASK) / INSTRUCTION_SIZE) as usize;
        let generation = mem.page_generation(page_base)?;
        let slot = cache_slot(page_base);

        let fetched = if let Some(line) = &mut self.pages[slot]
            && line.page_base == page_base
            && line.page.generation == generation
        {
            self.hits += 1;
            fetch_page_word(mem, page_base, &mut line.page, word_offset)?
        } else {
            self.misses += 1;
            let mut page = DecodedPage::new(generation);
            let fetched = fetch_page_word(mem, page_base, &mut page, word_offset)?;
            self.pages[slot] = Some(CacheLine { page_base, page });
            fetched
        };

        if fetched.decoded {
            self.word_decodes += 1;
        }
        Some(fetched.instr)
    }
}

impl Default for DecodeCache {
    fn default() -> Self {
        Self::new()
    }
}

struct FetchedInstr {
    instr: Instr,
    decoded: bool,
}

fn fetch_page_word(
    mem: &PhysicalMemory,
    page_base: u64,
    page: &mut DecodedPage,
    word_offset: usize,
) -> Option<FetchedInstr> {
    if let Some(instr) = page.instrs[word_offset] {
        return Some(FetchedInstr {
            instr,
            decoded: false,
        });
    }

    let instr = decode_page_word(mem, page_base, word_offset)?;
    page.instrs[word_offset] = Some(instr);
    Some(FetchedInstr {
        instr,
        decoded: true,
    })
}

fn decode_page_word(mem: &PhysicalMemory, page_base: u64, word_offset: usize) -> Option<Instr> {
    let addr = page_base + word_offset as u64 * INSTRUCTION_SIZE;
    let raw = mem.read_u32(addr)?;
    Some(decode(raw).unwrap_or_else(Instr::nop))
}

fn cache_slot(page_base: u64) -> usize {
    ((page_base / PAGE_SIZE) as usize) & (DECODE_CACHE_LINES - 1)
}

#[cfg(test)]
mod tests;
