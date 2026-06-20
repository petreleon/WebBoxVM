#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtAddr(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysRange {
    start: PhysAddr,
    len: u64,
}

impl PhysAddr {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn checked_add(self, offset: u64) -> Option<Self> {
        self.0.checked_add(offset).map(Self)
    }

    pub fn offset_from(self, base: Self) -> Option<u64> {
        self.0.checked_sub(base.0)
    }
}

impl VirtAddr {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl PhysRange {
    pub fn new(start: PhysAddr, len: u64) -> Option<Self> {
        start.get().checked_add(len)?;
        Some(Self { start, len })
    }

    pub const fn start(self) -> PhysAddr {
        self.start
    }

    pub const fn len(self) -> u64 {
        self.len
    }

    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    pub fn end_exclusive(self) -> PhysAddr {
        PhysAddr::new(self.start.get() + self.len)
    }

    pub fn contains(self, addr: PhysAddr) -> bool {
        self.start <= addr && addr < self.end_exclusive()
    }

    pub fn overlaps(self, other: Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        self.start < other.end_exclusive() && other.start < self.end_exclusive()
    }
}

impl From<PhysAddr> for u64 {
    fn from(value: PhysAddr) -> Self {
        value.get()
    }
}

impl From<VirtAddr> for u64 {
    fn from(value: VirtAddr) -> Self {
        value.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_and_virtual_addresses_are_distinct_newtypes() {
        let pa = PhysAddr::new(0x4000_0000);
        let va = VirtAddr::new(0xffff_8000_4000_0000);
        assert_ne!(pa.get(), va.get());
    }

    #[test]
    fn physical_address_arithmetic_is_checked() {
        assert_eq!(PhysAddr::new(10).checked_add(5), Some(PhysAddr::new(15)));
        assert_eq!(PhysAddr::new(u64::MAX).checked_add(1), None);
        assert_eq!(PhysAddr::new(15).offset_from(PhysAddr::new(10)), Some(5));
    }

    #[test]
    fn physical_ranges_are_half_open_and_overflow_checked() {
        let range = PhysRange::new(PhysAddr::new(10), 4).unwrap();
        assert!(range.contains(PhysAddr::new(10)));
        assert!(range.contains(PhysAddr::new(13)));
        assert!(!range.contains(PhysAddr::new(14)));
        assert_eq!(PhysRange::new(PhysAddr::new(u64::MAX), 1), None);
    }

    #[test]
    fn physical_range_overlap_respects_empty_ranges() {
        let left = PhysRange::new(PhysAddr::new(10), 4).unwrap();
        let right = PhysRange::new(PhysAddr::new(13), 4).unwrap();
        let touching = PhysRange::new(PhysAddr::new(14), 2).unwrap();
        let empty_at_start = PhysRange::new(PhysAddr::new(10), 0).unwrap();
        let empty_inside = PhysRange::new(PhysAddr::new(13), 0).unwrap();
        assert!(left.overlaps(right));
        assert!(!left.overlaps(touching));
        assert!(!left.overlaps(empty_at_start));
        assert!(!left.overlaps(empty_inside));
    }
}
