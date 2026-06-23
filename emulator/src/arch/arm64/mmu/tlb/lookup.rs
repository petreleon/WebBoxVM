use super::validate::*;
use super::*;

pub(super) const L1_TLB_PAGE_MASK: u64 = (L1_BLOCK_SIZE - 1) >> PAGE_SHIFT;
pub(super) const L2_TLB_PAGE_MASK: u64 = (L2_BLOCK_SIZE - 1) >> PAGE_SHIFT;

impl Tlb {
    pub(in crate::arch::arm64::mmu) fn lookup(
        &mut self,
        mem: &PhysicalMemory,
        va: u64,
        context: TlbContext,
    ) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        for key in lookup_keys(page) {
            let entry = &mut self.entries[tlb_index(key)];
            if read_entry_valid(entry, mem, page, context, self.epoch) {
                return Some(hit_pa(entry.pa_page, page, entry.page_mask, va));
            }
        }
        None
    }

    pub(in crate::arch::arm64::mmu) fn lookup_read_only(
        &self,
        mem: &PhysicalMemory,
        va: u64,
        context: TlbContext,
    ) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        for key in lookup_keys(page) {
            let entry = &self.entries[tlb_index(key)];
            if read_entry_valid_read_only(entry, mem, page, context, self.epoch) {
                return Some(hit_pa(entry.pa_page, page, entry.page_mask, va));
            }
        }
        None
    }

    pub(in crate::arch::arm64::mmu) fn lookup_write(
        &mut self,
        mem: &PhysicalMemory,
        va: u64,
        current_el: u8,
        context: TlbContext,
    ) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        for key in lookup_keys(page) {
            let entry = &mut self.write_entries[tlb_index(key)];
            if write_entry_valid(entry, mem, page, context, self.epoch)
                && (current_el != 0 || entry.el0_accessible)
            {
                return Some(hit_pa(entry.pa_page, page, entry.page_mask, va));
            }
        }
        None
    }
}

pub(super) fn block_page(page: u64, page_mask: u64) -> u64 {
    page & !page_mask
}

pub(super) fn tlb_index(page: u64) -> usize {
    (page & TLB_INDEX_MASK) as usize
}

fn lookup_keys(page: u64) -> [u64; 3] {
    [
        page,
        block_page(page, L2_TLB_PAGE_MASK),
        block_page(page, L1_TLB_PAGE_MASK),
    ]
}

fn hit_pa(pa_page: u64, page: u64, page_mask: u64, va: u64) -> u64 {
    ((pa_page | (page & page_mask)) << PAGE_SHIFT) | (va & PAGE_OFFSET_MASK)
}
