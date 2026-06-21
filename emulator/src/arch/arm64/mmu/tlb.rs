use super::*;

/// A single TLB entry at page granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TlbEntry {
    pub valid: bool,
    pub va_page: u64,
    pub pa_page: u64,
    context: TlbContext,
    desc_addr: u64,
    desc_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct WriteTlbEntry {
    valid: bool,
    va_page: u64,
    pa_page: u64,
    context: TlbContext,
    desc_addr: u64,
    desc_generation: u64,
    el0_accessible: bool,
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
}

impl Tlb {
    pub fn new() -> Self {
        Self {
            entries: vec![TlbEntry::default(); TLB_ENTRIES],
            write_entries: vec![WriteTlbEntry::default(); TLB_ENTRIES],
        }
    }

    pub(super) fn lookup(&self, mem: &PhysicalMemory, va: u64, context: TlbContext) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        let entry = &self.entries[idx];
        if self.entry_valid(entry, mem, page, context) {
            Some((entry.pa_page << PAGE_SHIFT) | (va & PAGE_OFFSET_MASK))
        } else {
            None
        }
    }

    pub(super) fn lookup_write(
        &self,
        mem: &PhysicalMemory,
        va: u64,
        current_el: u8,
        context: TlbContext,
    ) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        let entry = &self.write_entries[idx];
        let el_allowed = current_el != 0 || entry.el0_accessible;
        if self.write_entry_valid(entry, mem, page, context) && el_allowed {
            Some((entry.pa_page << PAGE_SHIFT) | (va & PAGE_OFFSET_MASK))
        } else {
            None
        }
    }

    pub(super) fn insert(
        &mut self,
        va: u64,
        pa: u64,
        context: TlbContext,
        desc_addr: u64,
        desc_generation: u64,
    ) {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        self.entries[idx] = TlbEntry {
            valid: true,
            va_page: page,
            pa_page: pa >> PAGE_SHIFT,
            context,
            desc_addr,
            desc_generation,
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
            el0_accessible,
        };
    }

    pub fn invalidate_all(&mut self) {
        for entry in &mut self.entries {
            entry.valid = false;
        }
        for entry in &mut self.write_entries {
            entry.valid = false;
        }
    }

    fn entry_valid(
        &self,
        entry: &TlbEntry,
        mem: &PhysicalMemory,
        page: u64,
        context: TlbContext,
    ) -> bool {
        entry.valid
            && entry.va_page == page
            && entry.context == context
            && descriptor_generation(mem, entry.desc_addr) == Some(entry.desc_generation)
    }

    fn write_entry_valid(
        &self,
        entry: &WriteTlbEntry,
        mem: &PhysicalMemory,
        page: u64,
        context: TlbContext,
    ) -> bool {
        entry.valid
            && entry.va_page == page
            && entry.context == context
            && descriptor_generation(mem, entry.desc_addr) == Some(entry.desc_generation)
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
}

pub(super) fn descriptor_generation(mem: &PhysicalMemory, desc_addr: u64) -> Option<u64> {
    mem.page_generation(desc_addr)
}
