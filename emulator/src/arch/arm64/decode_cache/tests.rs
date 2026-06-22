use super::*;
use crate::arch::arm64::Opcode;
use crate::constants::{INSTRUCTION_SIZE, RAM_BASE};

#[test]
fn cached_fetch_reuses_unchanged_page() {
    let mut mem = PhysicalMemory::new();
    let mut cache = DecodeCache::new();

    mem.write(RAM_BASE, 4, 0xd503_201f).unwrap();

    assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::Nop);
    assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::Nop);
    assert_eq!(cache.misses, 1);
    assert_eq!(cache.hits, 1);
}

#[test]
fn cached_fetch_redecodes_changed_word() {
    let mut mem = PhysicalMemory::new();
    let mut cache = DecodeCache::new();

    mem.write(RAM_BASE, 4, 0xd503_201f).unwrap();
    assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::Nop);

    mem.write(RAM_BASE, 4, 0x1400_0000).unwrap();

    assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::B);
    assert_eq!(cache.misses, 2);
}

#[test]
fn same_page_write_invalidates_cached_page() {
    let mut mem = PhysicalMemory::new();
    let mut cache = DecodeCache::new();

    mem.write(RAM_BASE, 4, 0xd503_201f).unwrap();
    assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::Nop);

    mem.write(RAM_BASE + INSTRUCTION_SIZE, 4, 0xd503_201f)
        .unwrap();
    assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::Nop);
    assert_eq!(cache.misses, 2);
}

#[test]
fn cache_miss_decodes_later_word_from_same_page() {
    let mut mem = PhysicalMemory::new();
    let mut cache = DecodeCache::new();
    let later = RAM_BASE + 17 * INSTRUCTION_SIZE;

    mem.write(later, 4, 0x1400_0000).unwrap();

    assert_eq!(cache.fetch(&mem, later).unwrap().op, Opcode::B);
    assert!(cache.fetch(&mem, RAM_BASE).is_some());
    assert_eq!(cache.misses, 1);
    assert_eq!(cache.hits, 1);
}

#[test]
fn direct_slot_collision_evictions_are_safe() {
    let mut mem = PhysicalMemory::new();
    let mut cache = DecodeCache::new();
    let first = RAM_BASE;
    let second = RAM_BASE + DECODE_CACHE_LINES as u64 * PAGE_SIZE;

    mem.write(first, 4, 0xd503_201f).unwrap();
    mem.write(second, 4, 0x1400_0000).unwrap();

    assert_eq!(cache.fetch(&mem, first).unwrap().op, Opcode::Nop);
    assert_eq!(cache.fetch(&mem, second).unwrap().op, Opcode::B);
    assert_eq!(cache.fetch(&mem, first).unwrap().op, Opcode::Nop);
    assert_eq!(cache.misses, 3);
}

#[test]
fn undecoded_words_are_tolerated_as_nops() {
    let mut mem = PhysicalMemory::new();
    let mut cache = DecodeCache::new();

    mem.write(RAM_BASE, 4, 0xffff_ffff).unwrap();

    assert_eq!(cache.fetch(&mem, RAM_BASE).unwrap().op, Opcode::Nop);
}

#[test]
fn zero_filled_code_page_does_not_allocate_guest_memory() {
    let mem = PhysicalMemory::new();
    let mut cache = DecodeCache::new();

    assert!(cache.fetch(&mem, RAM_BASE).is_some());
    assert_eq!(mem.allocated_pages(), 0);
    assert_eq!(cache.misses, 1);
}
