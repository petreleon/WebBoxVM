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
    memory_generation: u64,
    instrs: DecodedInstrs,
}

impl DecodedPage {
    fn new(generation: u64, memory_generation: u64) -> Self {
        Self {
            generation,
            memory_generation,
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
    pub page_validations: u64,
    pub word_decodes: u64,
}

impl DecodeCache {
    pub fn new() -> Self {
        Self {
            pages: vec![None; DECODE_CACHE_LINES],
            hits: 0,
            misses: 0,
            page_validations: 0,
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
        let memory_generation = mem.generation();
        let slot = cache_slot(page_base);

        let fetched = match cached_page_status(
            &mut self.pages[slot],
            mem,
            page_base,
            memory_generation,
            &mut self.page_validations,
        ) {
            CacheStatus::Hit => {
                let line = self.pages[slot].as_mut().expect("validated cache line");
                self.hits += 1;
                fetch_page_word(mem, page_base, &mut line.page, word_offset)?
            }
            CacheStatus::Miss(generation) => {
                self.misses += 1;
                let generation = generation.or_else(|| {
                    checked_page_generation(mem, page_base, &mut self.page_validations)
                })?;
                let mut page = DecodedPage::new(generation, memory_generation);
                let fetched = fetch_page_word(mem, page_base, &mut page, word_offset)?;
                self.pages[slot] = Some(CacheLine { page_base, page });
                fetched
            }
        };

        if fetched.1 {
            self.word_decodes += 1;
        }
        Some(fetched.0)
    }
}

impl Default for DecodeCache {
    fn default() -> Self {
        Self::new()
    }
}

enum CacheStatus {
    Hit,
    Miss(Option<u64>),
}

fn cached_page_status(
    line: &mut Option<CacheLine>,
    mem: &PhysicalMemory,
    page_base: u64,
    memory_generation: u64,
    validations: &mut u64,
) -> CacheStatus {
    let Some(line) = line else {
        return CacheStatus::Miss(None);
    };
    if line.page_base != page_base {
        return CacheStatus::Miss(None);
    }
    if line.page.memory_generation == memory_generation {
        return CacheStatus::Hit;
    }
    let Some(generation) = checked_page_generation(mem, page_base, validations) else {
        return CacheStatus::Miss(None);
    };
    if line.page.generation != generation {
        return CacheStatus::Miss(Some(generation));
    }
    line.page.memory_generation = memory_generation;
    CacheStatus::Hit
}

fn fetch_page_word(
    mem: &PhysicalMemory,
    page_base: u64,
    page: &mut DecodedPage,
    word_offset: usize,
) -> Option<(Instr, bool)> {
    if let Some(instr) = page.instrs[word_offset] {
        return Some((instr, false));
    }

    let instr = decode_page_word(mem, page_base, word_offset)?;
    page.instrs[word_offset] = Some(instr);
    Some((instr, true))
}

fn decode_page_word(mem: &PhysicalMemory, page_base: u64, word_offset: usize) -> Option<Instr> {
    let addr = page_base + word_offset as u64 * INSTRUCTION_SIZE;
    let raw = mem.read_u32(addr)?;
    Some(decode(raw).unwrap_or_else(Instr::nop))
}

fn checked_page_generation(
    mem: &PhysicalMemory,
    page_base: u64,
    validations: &mut u64,
) -> Option<u64> {
    *validations += 1;
    mem.page_generation(page_base)
}

fn cache_slot(page_base: u64) -> usize {
    ((page_base / PAGE_SIZE) as usize) & (DECODE_CACHE_LINES - 1)
}

#[cfg(test)]
mod tests;
