use super::*;

mod lookup;
#[cfg(test)]
mod tests;
mod validate;
use lookup::{block_page, tlb_index};

/// A single TLB entry for a page or block descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TlbEntry {
    pub valid: bool,
    pub va_page: u64,
    pub pa_page: u64,
    page_mask: u64,
    context: TlbContext,
    desc_addr: u64,
    desc_generation: u64,
    memory_generation: u64,
    epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct WriteTlbEntry {
    valid: bool,
    va_page: u64,
    pa_page: u64,
    page_mask: u64,
    context: TlbContext,
    desc_addr: u64,
    desc_generation: u64,
    memory_generation: u64,
    el0_accessible: bool,
    epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct TlbContext {
    pub(super) root: u64,
    pub(super) tcr: u64,
}

/// Direct-mapped software TLB with `TLB_ENTRIES` slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tlb {
    pub entries: Vec<TlbEntry>,
    write_entries: Vec<WriteTlbEntry>,
    epoch: u64,
}

impl Tlb {
    pub fn new() -> Self {
        Self {
            entries: vec![TlbEntry::default(); TLB_ENTRIES],
            write_entries: vec![WriteTlbEntry::default(); TLB_ENTRIES],
            epoch: 1,
        }
    }

    pub(super) fn insert(&mut self, va: u64, pa: u64, meta: TlbInsert) {
        let page = va >> PAGE_SHIFT;
        let page_base = block_page(page, meta.page_mask);
        let idx = tlb_index(page_base);
        self.entries[idx] = TlbEntry {
            valid: true,
            va_page: page_base,
            pa_page: block_page(pa >> PAGE_SHIFT, meta.page_mask),
            page_mask: meta.page_mask,
            context: meta.context,
            desc_addr: meta.desc_addr,
            desc_generation: meta.desc_generation,
            memory_generation: meta.memory_generation,
            epoch: self.epoch,
        };
    }

    pub(super) fn insert_write(&mut self, va: u64, pa: u64, meta: TlbInsert, el0_accessible: bool) {
        let page = va >> PAGE_SHIFT;
        let page_base = block_page(page, meta.page_mask);
        let idx = tlb_index(page_base);
        self.write_entries[idx] = WriteTlbEntry {
            valid: true,
            va_page: page_base,
            pa_page: block_page(pa >> PAGE_SHIFT, meta.page_mask),
            page_mask: meta.page_mask,
            context: meta.context,
            desc_addr: meta.desc_addr,
            desc_generation: meta.desc_generation,
            memory_generation: meta.memory_generation,
            el0_accessible,
            epoch: self.epoch,
        };
    }

    pub fn invalidate_all(&mut self) {
        if let Some(next) = self.epoch.checked_add(1) {
            self.epoch = next;
            return;
        }
        self.epoch = 1;
        for entry in &mut self.entries {
            entry.valid = false;
        }
        for entry in &mut self.write_entries {
            entry.valid = false;
        }
    }
}

impl Default for Tlb {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TlbInsert {
    pub(super) context: TlbContext,
    pub(super) desc_addr: u64,
    pub(super) desc_generation: u64,
    pub(super) memory_generation: u64,
    pub(super) page_mask: u64,
}

pub(super) fn descriptor_generation(mem: &PhysicalMemory, desc_addr: u64) -> Option<u64> {
    mem.page_generation(desc_addr)
}
