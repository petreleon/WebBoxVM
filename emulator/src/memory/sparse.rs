use crate::constants::*;
use std::collections::HashMap;

const MEMORY_PAGE_SIZE: usize = PAGE_SIZE as usize;
type Page = Box<[u8; MEMORY_PAGE_SIZE]>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SparseRegion {
    base: u64,
    size: u64,
    pages: HashMap<u64, Page>,
}

impl SparseRegion {
    pub(super) fn new(base: u64, size: u64) -> Self {
        Self {
            base,
            size,
            pages: HashMap::new(),
        }
    }

    pub(super) fn contains_range(&self, addr: u64, len: usize) -> bool {
        let Some(end) = addr.checked_add(len as u64) else {
            return false;
        };
        addr >= self.base && end <= self.base + self.size
    }

    pub(super) fn read_bytes(&self, addr: u64, dst: &mut [u8]) -> Option<()> {
        if !self.contains_range(addr, dst.len()) {
            return None;
        }

        let mut done = 0usize;
        while done < dst.len() {
            let current = addr + done as u64;
            let offset = current - self.base;
            let page_index = offset / PAGE_SIZE;
            let page_offset = (offset % PAGE_SIZE) as usize;
            let chunk = (dst.len() - done).min(MEMORY_PAGE_SIZE - page_offset);
            if let Some(page) = self.pages.get(&page_index) {
                dst[done..done + chunk].copy_from_slice(&page[page_offset..page_offset + chunk]);
            } else {
                dst[done..done + chunk].fill(0);
            }
            done += chunk;
        }

        Some(())
    }

    pub(super) fn write_bytes(&mut self, addr: u64, src: &[u8]) -> Option<()> {
        if !self.contains_range(addr, src.len()) {
            return None;
        }

        let mut done = 0usize;
        while done < src.len() {
            let current = addr + done as u64;
            let offset = current - self.base;
            let page_index = offset / PAGE_SIZE;
            let page_offset = (offset % PAGE_SIZE) as usize;
            let chunk = (src.len() - done).min(MEMORY_PAGE_SIZE - page_offset);
            let page = self
                .pages
                .entry(page_index)
                .or_insert_with(|| Box::new([0; MEMORY_PAGE_SIZE]));
            page[page_offset..page_offset + chunk].copy_from_slice(&src[done..done + chunk]);
            done += chunk;
        }

        Some(())
    }

    pub(super) fn allocated_pages(&self) -> usize {
        self.pages.len()
    }
}
