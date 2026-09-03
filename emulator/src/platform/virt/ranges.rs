use crate::constants::*;

const UART_FIXMAP_BASE: u64 = 0x090000;
const UART_FIXMAP_END: u64 = 0x091000;

pub(super) fn in_uart_range(addr: u64) -> bool {
    addr >= UART_BASE && addr < UART_END
}

pub(super) fn in_uart_fixmap_range(low: u64) -> bool {
    low >= UART_FIXMAP_BASE && low < UART_FIXMAP_END
}

pub(super) fn in_gicd_range(addr: u64) -> bool {
    addr >= GICD_BASE && addr < GICD_BASE + GICD_SIZE
}

pub(super) fn in_gicr_range(addr: u64) -> bool {
    addr >= GICR_BASE && addr < GICR_BASE + GICR_SIZE
}

pub(super) fn in_virtio_blk_range(addr: u64) -> bool {
    addr >= VIRTIO_BLK_BASE && addr < VIRTIO_BLK_END
}

pub(super) fn in_virtio_disk_range(addr: u64) -> bool {
    addr >= VIRTIO_DISK_BASE && addr < VIRTIO_DISK_END
}

pub(super) fn in_virtio_net_range(addr: u64) -> bool {
    addr >= VIRTIO_NET_BASE && addr < VIRTIO_NET_END
}

pub(super) fn in_virtio_gpu_range(addr: u64) -> bool {
    addr >= VIRTIO_GPU_BASE && addr < VIRTIO_GPU_END
}

pub(super) fn overlaps_device_range(addr: u64, len: usize) -> bool {
    if len == 0 {
        return false;
    }
    let Some(range_end) = addr.checked_add(len as u64) else {
        return true;
    };
    if addr >= LOW_REGION_END {
        return false;
    }

    range_overlaps(addr, range_end, UART_BASE, UART_END)
        || range_overlaps(addr, range_end, GICD_BASE, GICD_BASE + GICD_SIZE)
        || range_overlaps(addr, range_end, GICR_BASE, GICR_BASE + GICR_SIZE)
        || range_overlaps(addr, range_end, VIRTIO_BLK_BASE, VIRTIO_BLK_END)
        || range_overlaps(addr, range_end, VIRTIO_DISK_BASE, VIRTIO_DISK_END)
        || range_overlaps(addr, range_end, VIRTIO_NET_BASE, VIRTIO_NET_END)
        || range_overlaps(addr, range_end, VIRTIO_GPU_BASE, VIRTIO_GPU_END)
}

pub(super) fn is_printable_or_control(b: u8) -> bool {
    matches!(b, b' '..=b'~' | b'\n' | b'\r')
}

fn range_overlaps(addr: u64, range_end: u64, base: u64, end: u64) -> bool {
    addr < end && base < range_end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_memory_ranges_skip_device_overlap_scan() {
        assert!(!overlaps_device_range(RAM_BASE, 8));
        assert!(!overlaps_device_range(EFI_REGION_BASE, 0x1000));
        assert!(!overlaps_device_range(RAM_END, 4));
    }

    #[test]
    fn device_overlap_keeps_edges_and_overflow_conservative() {
        assert!(!overlaps_device_range(UART_BASE, 0));
        assert!(!overlaps_device_range(VIRTIO_BLK_BASE - 1, 1));
        assert!(overlaps_device_range(VIRTIO_BLK_BASE - 1, 2));
        assert!(overlaps_device_range(UART_END - 1, 1));
        assert!(!overlaps_device_range(UART_END, 1));
        assert!(overlaps_device_range(u64::MAX - 1, 8));
    }
}
