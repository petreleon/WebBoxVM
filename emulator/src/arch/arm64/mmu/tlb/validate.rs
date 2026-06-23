use super::lookup::block_page;
use super::*;

pub(super) fn read_entry_valid(
    entry: &mut TlbEntry,
    mem: &PhysicalMemory,
    page: u64,
    context: TlbContext,
    epoch: u64,
) -> bool {
    entry.valid
        && entry.epoch == epoch
        && entry.va_page == block_page(page, entry.page_mask)
        && entry.context == context
        && read_descriptor_current(entry, mem)
}

pub(super) fn read_entry_valid_read_only(
    entry: &TlbEntry,
    mem: &PhysicalMemory,
    page: u64,
    context: TlbContext,
    epoch: u64,
) -> bool {
    entry.valid
        && entry.epoch == epoch
        && entry.va_page == block_page(page, entry.page_mask)
        && entry.context == context
        && read_descriptor_current_read_only(entry, mem)
}

pub(super) fn write_entry_valid(
    entry: &mut WriteTlbEntry,
    mem: &PhysicalMemory,
    page: u64,
    context: TlbContext,
    epoch: u64,
) -> bool {
    entry.valid
        && entry.epoch == epoch
        && entry.va_page == block_page(page, entry.page_mask)
        && entry.context == context
        && write_descriptor_current(entry, mem)
}

fn read_descriptor_current(entry: &mut TlbEntry, mem: &PhysicalMemory) -> bool {
    let memory_generation = mem.generation();
    if entry.memory_generation == memory_generation {
        return true;
    }
    if descriptor_generation(mem, entry.desc_addr) != Some(entry.desc_generation) {
        return false;
    }
    entry.memory_generation = memory_generation;
    true
}

fn read_descriptor_current_read_only(entry: &TlbEntry, mem: &PhysicalMemory) -> bool {
    entry.memory_generation == mem.generation()
        || descriptor_generation(mem, entry.desc_addr) == Some(entry.desc_generation)
}

fn write_descriptor_current(entry: &mut WriteTlbEntry, mem: &PhysicalMemory) -> bool {
    let memory_generation = mem.generation();
    if entry.memory_generation == memory_generation {
        return true;
    }
    if descriptor_generation(mem, entry.desc_addr) != Some(entry.desc_generation) {
        return false;
    }
    entry.memory_generation = memory_generation;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_validation_refreshes_after_unrelated_write() {
        let mut mem = PhysicalMemory::new();
        let context = TlbContext { root: 0, tcr: 0 };
        let desc_addr = RAM_BASE;

        mem.write(desc_addr, 8, 0x4000_3001).unwrap();
        let mut entry = TlbEntry {
            valid: true,
            va_page: 1,
            pa_page: 2,
            page_mask: 0,
            context,
            desc_addr,
            desc_generation: mem.page_generation(desc_addr).unwrap(),
            memory_generation: mem.generation(),
            epoch: 3,
        };

        mem.write(RAM_BASE + PAGE_SIZE, 8, 0xAA55).unwrap();
        assert!(read_entry_valid(&mut entry, &mem, 1, context, 3));
        assert_eq!(entry.memory_generation, mem.generation());

        mem.write(desc_addr, 8, 0x4000_5001).unwrap();
        assert!(!read_entry_valid(&mut entry, &mem, 1, context, 3));
    }

    #[test]
    fn read_only_validation_rechecks_after_memory_changes() {
        let mut mem = PhysicalMemory::new();
        let context = TlbContext { root: 0, tcr: 0 };
        let desc_addr = RAM_BASE;

        mem.write(desc_addr, 8, 0x4000_3001).unwrap();
        let entry = TlbEntry {
            valid: true,
            va_page: 1,
            pa_page: 2,
            page_mask: 0,
            context,
            desc_addr,
            desc_generation: mem.page_generation(desc_addr).unwrap(),
            memory_generation: mem.generation(),
            epoch: 3,
        };

        assert!(read_entry_valid_read_only(&entry, &mem, 1, context, 3));
        mem.write(RAM_BASE + PAGE_SIZE, 8, 0xAA55).unwrap();
        assert!(read_entry_valid_read_only(&entry, &mem, 1, context, 3));
        mem.write(desc_addr, 8, 0x4000_5001).unwrap();
        assert!(!read_entry_valid_read_only(&entry, &mem, 1, context, 3));
    }
}
