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

pub(super) fn is_printable_or_control(b: u8) -> bool {
    matches!(b, b' '..=b'~' | b'\n' | b'\r')
}
