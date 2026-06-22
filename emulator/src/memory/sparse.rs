use crate::constants::*;

const MEMORY_PAGE_SIZE: usize = PAGE_SIZE as usize;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Page {
    bytes: Box<[u8; MEMORY_PAGE_SIZE]>,
    generation: u64,
}

impl Page {
    fn new() -> Self {
        Self {
            bytes: Box::new([0; MEMORY_PAGE_SIZE]),
            generation: 0,
        }
    }

    fn bump_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SparseRegion {
    base: u64,
    size: u64,
    pages: Vec<Option<Page>>,
}

impl SparseRegion {
    pub(super) fn new(base: u64, size: u64) -> Self {
        Self {
            base,
            size,
            pages: vec![None; (size / PAGE_SIZE) as usize],
        }
    }

    pub(super) fn contains_range(&self, addr: u64, len: usize) -> bool {
        let Some(end) = addr.checked_add(len as u64) else {
            return false;
        };
        addr >= self.base && end <= self.base + self.size
    }

    pub(super) fn contains_addr(&self, addr: u64) -> bool {
        addr.wrapping_sub(self.base) < self.size
    }

    pub(super) fn read_bytes_in_region(&self, addr: u64, dst: &mut [u8]) {
        debug_assert!(self.contains_range(addr, dst.len()));

        let mut done = 0usize;
        while done < dst.len() {
            let current = addr + done as u64;
            let offset = current - self.base;
            let page_index = (offset / PAGE_SIZE) as usize;
            let page_offset = (offset % PAGE_SIZE) as usize;
            let chunk = (dst.len() - done).min(MEMORY_PAGE_SIZE - page_offset);
            if let Some(page) = &self.pages[page_index] {
                dst[done..done + chunk]
                    .copy_from_slice(&page.bytes[page_offset..page_offset + chunk]);
            } else {
                dst[done..done + chunk].fill(0);
            }
            done += chunk;
        }
    }

    pub(super) fn read_array<const N: usize>(&self, addr: u64) -> [u8; N] {
        debug_assert!(self.contains_range(addr, N));

        let offset = addr - self.base;
        let page_index = (offset / PAGE_SIZE) as usize;
        let page_offset = (offset % PAGE_SIZE) as usize;
        let mut bytes = [0; N];

        if page_offset + N > MEMORY_PAGE_SIZE {
            self.read_bytes_in_region(addr, &mut bytes);
            return bytes;
        }

        if let Some(page) = &self.pages[page_index] {
            bytes.copy_from_slice(&page.bytes[page_offset..page_offset + N]);
        }
        bytes
    }

    pub(super) fn page_generation(&self, addr: u64) -> u64 {
        debug_assert!(self.contains_addr(addr));
        let page_index = ((addr - self.base) / PAGE_SIZE) as usize;
        self.pages[page_index]
            .as_ref()
            .map_or(0, |page| page.generation)
    }

    pub(super) fn write_bytes_in_region(&mut self, addr: u64, src: &[u8]) {
        debug_assert!(self.contains_range(addr, src.len()));

        let mut done = 0usize;
        while done < src.len() {
            let current = addr + done as u64;
            let offset = current - self.base;
            let page_index = (offset / PAGE_SIZE) as usize;
            let page_offset = (offset % PAGE_SIZE) as usize;
            let chunk = (src.len() - done).min(MEMORY_PAGE_SIZE - page_offset);
            let page = self.pages[page_index].get_or_insert_with(Page::new);
            page.bytes[page_offset..page_offset + chunk].copy_from_slice(&src[done..done + chunk]);
            page.bump_generation();
            done += chunk;
        }
    }

    pub(super) fn allocated_pages(&self) -> usize {
        self.pages.iter().filter(|page| page.is_some()).count()
    }
}
