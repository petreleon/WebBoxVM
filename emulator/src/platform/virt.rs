//! Virt board routing — maps physical addresses to RAM and MMIO devices.
//!
//! Physical addresses are checked in this order:
//!   1. UART (PL011 at 0x0900_0000)
//!   2. GICv3 (distributor at 0x0800_0000, redistributor at 0x080A_0000)
//!   3. Physical memory (fallback for RAM, EFI, and low region)

use crate::constants::*;
use crate::devices::gicv3::Gicv3;
use crate::devices::pl011::Pl011Uart;
use crate::devices::virtio_blk::VirtioBlk;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_net::VirtioNet;
use crate::memory::PhysicalMemory;

mod access;
mod construction;
mod exclusive;
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
    pub virtio_gpu: VirtioGpu,
    uart_rx_refresh_needed: bool,
    memory_writes: Vec<exclusive::MemoryWrite>,
    // Virtio completions can touch several discontiguous guest buffers. A
    // global marker is both cheaper to propagate and architecturally safe:
    // exclusive monitors are allowed to fail spuriously.
    dma_write_during_instruction: bool,
    external_dma_write_pending: bool,
}

impl SystemBus {
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

        if addr >= LOW_REGION_END && addr < KERNEL_VA_BASE {
            return self.mem.read(addr, size);
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
        if in_virtio_gpu_range(addr) {
            return self.virtio_gpu.read(addr - VIRTIO_GPU_BASE, size);
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

        if addr >= LOW_REGION_END && addr < KERNEL_VA_BASE {
            if self.mem.write(addr, size, value).is_some() {
                self.record_memory_write(addr, size as u64);
            }
            return;
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
                self.record_dma_write();
                self.gic.set_pending(VIRTIO_BLK_IRQ_ID);
            }
            return;
        } else if in_virtio_disk_range(addr) {
            if self
                .virtio_disk
                .write(&mut self.mem, addr - VIRTIO_DISK_BASE, value, size)
            {
                self.record_dma_write();
                self.gic.set_pending(VIRTIO_DISK_IRQ_ID);
            }
            return;
        } else if in_virtio_net_range(addr) {
            if self
                .virtio_net
                .write(&mut self.mem, addr - VIRTIO_NET_BASE, value, size)
            {
                self.record_dma_write();
                self.gic.set_pending(VIRTIO_NET_IRQ_ID);
            }
            return;
        } else if in_virtio_gpu_range(addr) {
            if self
                .virtio_gpu
                .write(&mut self.mem, addr - VIRTIO_GPU_BASE, value, size)
            {
                self.record_dma_write();
                self.gic.set_pending(VIRTIO_GPU_IRQ_ID);
            }
            return;
        }
        if self.mem.write(addr, size, value).is_some() {
            self.record_memory_write(addr, size as u64);
        }
    }
}

#[cfg(test)]
mod tests;
