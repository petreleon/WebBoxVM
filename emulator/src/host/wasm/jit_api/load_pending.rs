use crate::host::wasm::JitPendingStore;

pub(super) fn pending_store_byte(pending_stores: &[JitPendingStore], pa: u64) -> Option<u64> {
    pending_stores.iter().rev().find_map(|store| {
        let offset = pa.checked_sub(store.pa)?;
        if offset < store.len as u64 {
            Some(store.bytes[offset as usize] as u64)
        } else {
            None
        }
    })
}

pub(super) fn pending_stores_overlap_range(
    pending_stores: &[JitPendingStore],
    pa: u64,
    len: usize,
) -> bool {
    if len == 0 || pending_stores.is_empty() {
        return false;
    }
    let Some(end) = pa.checked_add(len as u64) else {
        return true;
    };
    pending_stores
        .iter()
        .any(|store| pending_store_overlaps(store, pa, end))
}

fn pending_store_overlaps(store: &JitPendingStore, pa: u64, end: u64) -> bool {
    let Some(store_end) = store.pa.checked_add(store.len as u64) else {
        return true;
    };
    store.pa < end && pa < store_end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::RAM_BASE;

    #[test]
    fn pending_store_overlap_respects_range_edges() {
        let stores = [JitPendingStore::new(RAM_BASE + 0x20, &[1, 2, 3, 4])];

        assert!(!pending_stores_overlap_range(
            &stores,
            RAM_BASE + 0x10,
            0x10
        ));
        assert!(pending_stores_overlap_range(&stores, RAM_BASE + 0x10, 0x11));
        assert!(pending_stores_overlap_range(&stores, RAM_BASE + 0x23, 1));
        assert!(!pending_stores_overlap_range(&stores, RAM_BASE + 0x24, 8));
    }

    #[test]
    fn pending_store_overlap_is_conservative_on_overflow() {
        let stores = [JitPendingStore::new(u64::MAX - 1, &[1, 2])];

        assert!(pending_stores_overlap_range(&stores, u64::MAX - 1, 8));
    }
}
