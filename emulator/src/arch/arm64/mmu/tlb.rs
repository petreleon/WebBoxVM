use super::*;

/// A single TLB entry at page granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TlbEntry {
    pub valid: bool,
    pub va_page: u64,
    pub pa_page: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct WriteTlbEntry {
    valid: bool,
    va_page: u64,
    pa_page: u64,
    el0_accessible: bool,
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

    pub fn lookup(&self, va: u64) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        let entry = &self.entries[idx];
        if entry.valid && entry.va_page == page {
            Some((entry.pa_page << PAGE_SHIFT) | (va & PAGE_OFFSET_MASK))
        } else {
            None
        }
    }

    pub fn lookup_write(&self, va: u64, current_el: u8) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        let entry = &self.write_entries[idx];
        let el_allowed = current_el != 0 || entry.el0_accessible;
        if entry.valid && entry.va_page == page && el_allowed {
            Some((entry.pa_page << PAGE_SHIFT) | (va & PAGE_OFFSET_MASK))
        } else {
            None
        }
    }

    pub fn insert(&mut self, va: u64, pa: u64) {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        self.entries[idx] = TlbEntry {
            valid: true,
            va_page: page,
            pa_page: pa >> PAGE_SHIFT,
        };
    }

    pub fn insert_write(&mut self, va: u64, pa: u64, el0_accessible: bool) {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        self.write_entries[idx] = WriteTlbEntry {
            valid: true,
            va_page: page,
            pa_page: pa >> PAGE_SHIFT,
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
}

impl Default for Tlb {
    fn default() -> Self {
        Self::new()
    }
}
