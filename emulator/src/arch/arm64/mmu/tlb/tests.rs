use super::lookup::{L2_TLB_PAGE_MASK, block_page};
use super::*;

#[test]
fn l2_block_read_entry_covers_neighbor_pages() {
    let mut mem = PhysicalMemory::new();
    let mut tlb = Tlb::new();
    let context = TlbContext { root: 7, tcr: 11 };
    let desc_addr = RAM_BASE;
    let va = 0xFFFF_FF80_0000_1000;
    let pa = 0x4000_1000;

    mem.write(desc_addr, 8, 0x4000_0001).unwrap();
    tlb.insert(va, pa, insert_meta(&mem, context, desc_addr));

    assert_eq!(
        tlb.lookup(&mem, va + PAGE_SIZE, context),
        Some(pa + PAGE_SIZE)
    );
}

#[test]
fn l2_block_write_entry_covers_neighbor_pages() {
    let mut mem = PhysicalMemory::new();
    let mut tlb = Tlb::new();
    let context = TlbContext { root: 13, tcr: 17 };
    let desc_addr = RAM_BASE;
    let va = 0xFFFF_FF80_0020_3000;
    let pa = 0x4020_3000;

    mem.write(desc_addr, 8, 0x4020_0001).unwrap();
    tlb.insert_write(va, pa, insert_meta(&mem, context, desc_addr), true);

    assert_eq!(
        tlb.lookup_write(&mem, va + PAGE_SIZE, 0, context),
        Some(pa + PAGE_SIZE)
    );
}

#[test]
fn l2_block_entry_rechecks_descriptor_generation() {
    let mut mem = PhysicalMemory::new();
    let mut tlb = Tlb::new();
    let context = TlbContext { root: 19, tcr: 23 };
    let desc_addr = RAM_BASE;
    let va = 0xFFFF_FF80_0040_1000;
    let pa = 0x4040_1000;

    mem.write(desc_addr, 8, 0x4040_0001).unwrap();
    tlb.insert(va, pa, insert_meta(&mem, context, desc_addr));
    mem.write(desc_addr, 8, 0x4060_0001).unwrap();

    assert_eq!(tlb.lookup(&mem, va + PAGE_SIZE, context), None);
}

fn insert_meta(mem: &PhysicalMemory, context: TlbContext, desc_addr: u64) -> TlbInsert {
    TlbInsert {
        context,
        desc_addr,
        desc_generation: mem.page_generation(desc_addr).unwrap(),
        memory_generation: mem.generation(),
        page_mask: L2_TLB_PAGE_MASK,
    }
}

#[test]
fn block_page_masks_low_pages() {
    assert_eq!(block_page(0x12345, L2_TLB_PAGE_MASK), 0x12200);
}
