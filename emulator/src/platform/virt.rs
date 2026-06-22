//! Virt board routing — maps physical addresses to RAM and MMIO devices.
//!
//! Physical addresses are checked in this order:
//!   1. UART (PL011 at 0x0900_0000)
//!   2. GICv3 (distributor at 0x0800_0000, redistributor at 0x080A_0000)
//!   3. Physical memory (fallback for RAM, EFI, and low region)

use crate::api::{AccessWidth, PhysAddr};
use crate::constants::*;
use crate::devices::gicv3::Gicv3;
use crate::devices::pl011::Pl011Uart;
use crate::devices::virtio_blk::VirtioBlk;
use crate::devices::virtio_net::VirtioNet;
use crate::memory::PhysicalMemory;

mod interrupts;
mod ranges;

use interrupts::gicd_clear_pending_touches_uart;
use ranges::*;

pub struct SystemBus {
    pub mem: PhysicalMemory,
    pub uart: Pl011Uart,
    pub gic: Gicv3,
    pub virtio_blk: VirtioBlk,
    pub virtio_disk: VirtioBlk,
    pub virtio_net: VirtioNet,
    uart_rx_refresh_needed: bool,
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
            virtio_net: VirtioNet::new(),
            uart_rx_refresh_needed: false,
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
        if in_virtio_net_range(addr) {
            return self.virtio_net.read(addr - VIRTIO_NET_BASE, size);
        }
        self.mem.read(addr, size)
    }

    pub fn read_phys(&mut self, addr: PhysAddr, width: AccessWidth) -> Option<u64> {
        self.read(addr.get(), width.bytes())
    }

    pub fn write_bytes(&mut self, addr: u64, bytes: &[u8]) -> Option<()> {
        if addr < LOW_REGION_END && overlaps_device_range(addr, bytes.len()) {
            return None;
        }
        self.mem.write_bytes(addr, bytes)
    }

    pub fn overlaps_device_range(&self, addr: u64, len: usize) -> bool {
        overlaps_device_range(addr, len)
    }

    pub fn inject_network_frame(&mut self, frame: &[u8]) {
        if self.virtio_net.inject_rx_frame(&mut self.mem, frame) {
            self.gic.set_pending(VIRTIO_NET_IRQ_ID);
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
            if addr - UART_BASE == UART_IMSC_OFFSET {
                self.mark_uart_rx_refresh_needed();
            }
            return;
        } else if in_gicd_range(addr) {
            let clears_uart = gicd_clear_pending_touches_uart(addr, value);
            self.gic.gicd_write(addr - GICD_BASE, value, size);
            if clears_uart {
                self.mark_uart_rx_refresh_needed();
            }
            return;
        } else if in_gicr_range(addr) {
            self.gic.gicr_write(addr - GICR_BASE, value, size);
            return;
        } else if in_virtio_blk_range(addr) {
            if self
                .virtio_blk
                .write(&mut self.mem, addr - VIRTIO_BLK_BASE, value, size)
            {
                self.gic.set_pending(VIRTIO_BLK_IRQ_ID);
            }
            return;
        } else if in_virtio_disk_range(addr) {
            if self
                .virtio_disk
                .write(&mut self.mem, addr - VIRTIO_DISK_BASE, value, size)
            {
                self.gic.set_pending(VIRTIO_DISK_IRQ_ID);
            }
            return;
        } else if in_virtio_net_range(addr) {
            if self
                .virtio_net
                .write(&mut self.mem, addr - VIRTIO_NET_BASE, value, size)
            {
                self.gic.set_pending(VIRTIO_NET_IRQ_ID);
            }
            return;
        }
        self.mem.write(addr, size, value);
    }

    pub fn write_phys(&mut self, addr: PhysAddr, width: AccessWidth, value: u64) {
        self.write(addr.get(), width.bytes(), value);
    }
}

#[cfg(test)]
mod tests;
