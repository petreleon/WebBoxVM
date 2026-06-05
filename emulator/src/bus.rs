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

mod ranges;

use ranges::*;

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

    pub fn refresh_interrupts(&mut self) {
        if self.uart.masked_rx_interrupt_pending() {
            self.gic.set_pending(PL011_UART_IRQ_ID);
        }
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

#[cfg(test)]
mod tests;
