use super::*;

mod validate;
use validate::*;

/// A single TLB entry at page granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TlbEntry {
    pub valid: bool,
    pub va_page: u64,
    pub pa_page: u64,
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

    pub(super) fn lookup(
        &mut self,
        mem: &PhysicalMemory,
        va: u64,
        context: TlbContext,
    ) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        let entry = &mut self.entries[idx];
        if read_entry_valid(entry, mem, page, context, self.epoch) {
            Some((entry.pa_page << PAGE_SHIFT) | (va & PAGE_OFFSET_MASK))
        } else {
            None
        }
    }

    pub(super) fn lookup_read_only(
        &self,
        mem: &PhysicalMemory,
        va: u64,
        context: TlbContext,
    ) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        let entry = &self.entries[idx];
        if read_entry_valid_read_only(entry, mem, page, context, self.epoch) {
            Some((entry.pa_page << PAGE_SHIFT) | (va & PAGE_OFFSET_MASK))
        } else {
            None
        }
    }

    pub(super) fn lookup_write(
        &mut self,
        mem: &PhysicalMemory,
        va: u64,
        current_el: u8,
        context: TlbContext,
    ) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        let entry = &mut self.write_entries[idx];
        let el_allowed = current_el != 0 || entry.el0_accessible;
        if write_entry_valid(entry, mem, page, context, self.epoch) && el_allowed {
            Some((entry.pa_page << PAGE_SHIFT) | (va & PAGE_OFFSET_MASK))
        } else {
            None
        }
    }

    pub(super) fn insert(&mut self, va: u64, pa: u64, meta: TlbInsert) {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        self.entries[idx] = TlbEntry {
            valid: true,
            va_page: page,
            pa_page: pa >> PAGE_SHIFT,
            context: meta.context,
            desc_addr: meta.desc_addr,
            desc_generation: meta.desc_generation,
            memory_generation: meta.memory_generation,
            epoch: self.epoch,
        };
    }

    pub(super) fn insert_write(&mut self, va: u64, pa: u64, meta: TlbInsert, el0_accessible: bool) {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        self.write_entries[idx] = WriteTlbEntry {
            valid: true,
            va_page: page,
            pa_page: pa >> PAGE_SHIFT,
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
}

pub(super) fn descriptor_generation(mem: &PhysicalMemory, desc_addr: u64) -> Option<u64> {
    mem.page_generation(desc_addr)
}
