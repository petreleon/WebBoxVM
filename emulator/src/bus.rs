//! MMIO dispatch table — routes physical addresses to the correct device handler.
//!
//! Physical addresses are checked in this order:
//!   1. UART (PL011 at 0x0900_0000)
//!   2. GICv3 (distributor at 0x0800_0000, redistributor at 0x080A_0000)
//!   3. Physical memory (fallback for RAM, EFI, and low region)

use crate::constants::*;
use crate::devices::gicv3::Gicv3;
use crate::devices::pl011::Pl011Uart;
use crate::devices::virtio_blk::VirtioBlk;
use crate::memory::PhysicalMemory;

/// Mask for the bottom 21 bits of an address, used for UART fixmap remapping.
const FIXMAP_LOW_MASK: u64 = 0x001F_FFFF;

/// Lower 16 bits of the UART fixmap address (0x09_0000 within a 2 MiB block).
const UART_FIXMAP_BASE: u64 = 0x090000;

/// Upper bound of the UART fixmap address (0x09_1000 = UART_FIXMAP_BASE + 4 KiB page).
const UART_FIXMAP_END: u64 = 0x091000;

/// Mask for the bottom 12 bits — in-page offset.
const PAGE_OFFSET_MASK: u64 = 0xFFF;

pub struct SystemBus {
    pub mem: PhysicalMemory,
    pub uart: Pl011Uart,
    pub gic: Gicv3,
    pub virtio_blk: VirtioBlk,
    pub virtio_disk: VirtioBlk,
}

impl SystemBus {
    pub fn new() -> Self {
        Self {
            mem: PhysicalMemory::new(),
            uart: Pl011Uart::new(),
            gic: Gicv3::new(),
            virtio_blk: VirtioBlk::new(),
            virtio_disk: VirtioBlk::writable_sparse(
                crate::devices::virtio_blk::DEFAULT_SPARSE_DISK_SIZE,
                b"webboxvm-disk\0",
            ),
        }
    }

    pub fn read(&mut self, addr: u64, size: u8) -> Option<u64> {
        // Redirect fixmap kernel VA UART reads to the correct device.
        // Only applies to kernel VAs (>= 0xffff000000000000), not physical addresses.
        if addr >= KERNEL_VA_BASE {
            let low = addr & FIXMAP_LOW_MASK;
            if in_uart_fixmap_range(low) {
                let uart_offset = addr & PAGE_OFFSET_MASK;
                return self.uart.read(UART_BASE | uart_offset, size);
            }
        }

        // Standard MMIO dispatch
        if in_uart_range(addr) {
            return self.uart.read(addr, size);
        }
        if in_gicd_range(addr) {
            return self.gic.gicd_read(addr - GICD_BASE, size);
        }
        if in_gicr_range(addr) {
            return self.gic.gicr_read(addr - GICR_BASE, size);
        }
        if in_virtio_blk_range(addr) {
            return self.virtio_blk.read(addr - VIRTIO_BLK_BASE, size);
        }
        if in_virtio_disk_range(addr) {
            return self.virtio_disk.read(addr - VIRTIO_DISK_BASE, size);
        }
        self.mem.read(addr, size)
    }

    pub fn write(&mut self, addr: u64, size: u8, value: u64) {
        // Redirect fixmap kernel VA UART writes to the correct device.
        // Only applies to kernel VAs (>= 0xffff000000000000), not physical addresses.
        if addr >= KERNEL_VA_BASE {
            let low = addr & FIXMAP_LOW_MASK;
            if in_uart_fixmap_range(low) && size == 1 && is_printable_or_control(value as u8) {
                let uart_offset = addr & PAGE_OFFSET_MASK;
                self.uart.write(UART_BASE | uart_offset, size, value);
                let _ = self.mem.write(addr, size, value);
                return;
            }
        }

        // Standard MMIO dispatch
        if in_uart_range(addr) {
            self.uart.write(addr, size, value);
            // Trace: kernel wrote to the UART physical address
        } else if in_gicd_range(addr) {
            self.gic.gicd_write(addr - GICD_BASE, value, size);
        } else if in_gicr_range(addr) {
            self.gic.gicr_write(addr - GICR_BASE, value, size);
        } else if in_virtio_blk_range(addr) {
            if self
                .virtio_blk
                .write(&mut self.mem, addr - VIRTIO_BLK_BASE, value, size)
            {
                self.gic.set_pending(VIRTIO_BLK_IRQ_ID);
            }
        } else if in_virtio_disk_range(addr) {
            if self
                .virtio_disk
                .write(&mut self.mem, addr - VIRTIO_DISK_BASE, value, size)
            {
                self.gic.set_pending(VIRTIO_DISK_IRQ_ID);
            }
        }
        self.mem.write(addr, size, value);
    }
}

// ── Address range predicates ──

fn in_uart_range(addr: u64) -> bool {
    addr >= UART_BASE && addr < UART_END
}

fn in_uart_fixmap_range(low: u64) -> bool {
    low >= UART_FIXMAP_BASE && low < UART_FIXMAP_END
}

fn in_gicd_range(addr: u64) -> bool {
    addr >= GICD_BASE && addr < GICD_BASE + GICD_SIZE
}

fn in_gicr_range(addr: u64) -> bool {
    addr >= GICR_BASE && addr < GICR_BASE + GICR_SIZE
}

fn in_virtio_blk_range(addr: u64) -> bool {
    addr >= VIRTIO_BLK_BASE && addr < VIRTIO_BLK_END
}

fn in_virtio_disk_range(addr: u64) -> bool {
    addr >= VIRTIO_DISK_BASE && addr < VIRTIO_DISK_END
}

/// Returns true for printable ASCII (0x20–0x7E), newline (0x0A), or CR (0x0D).
fn is_printable_or_control(b: u8) -> bool {
    matches!(b, b' '..=b'~' | b'\n' | b'\r')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uart_priority_over_ram() {
        let mut bus = SystemBus::new();
        bus.write(UART_BASE, 1, b'A' as u64);
        assert_eq!(bus.uart.output_string(), "A");
    }

    #[test]
    fn ram_read_write() {
        let mut bus = SystemBus::new();
        bus.write(RAM_BASE, 8, 0xDEADBEEF);
        assert_eq!(bus.read(RAM_BASE, 8), Some(0xDEADBEEF));
    }

    #[test]
    fn second_virtio_disk_has_own_mmio_window() {
        let mut bus = SystemBus::new();

        assert_eq!(bus.read(VIRTIO_BLK_BASE, 4), Some(0x7472_6976));
        assert_eq!(bus.read(VIRTIO_DISK_BASE, 4), Some(0x7472_6976));
    }
}
