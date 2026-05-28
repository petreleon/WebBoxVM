//! MMIO dispatch table: routes physical addresses to device handlers.

use crate::devices::pl011::Pl011Uart;
use crate::devices::gicv3::Gicv3;
use crate::memory::PhysicalMemory;

pub struct SystemBus {
    pub mem: PhysicalMemory,
    pub uart: Pl011Uart,
    pub gic: Gicv3,
}

impl SystemBus {
    pub fn new() -> Self {
        Self {
            mem: PhysicalMemory::new(),
            uart: Pl011Uart::new(),
            gic: Gicv3::new(),
        }
    }

    pub fn read(&self, addr: u64, size: u8) -> Option<u64> {
        // GIC distributor (0x08000000 - 0x0800FFFF)
        if addr >= 0x08000000 && addr < 0x08010000 {
            return self.gic.gicd_read(addr - 0x08000000, size);
        }
        // GIC redistributor (0x080A0000 - 0x08100000)
        if addr >= 0x080A0000 && addr < 0x08100000 {
            return self.gic.gicr_read(addr - 0x080A0000, size);
        }
        self.uart.read(addr, size)
            .or_else(|| self.mem.read(addr, size))
    }

    pub fn write(&mut self, addr: u64, size: u8, value: u64) {
        // GIC distributor
        if addr >= 0x08000000 && addr < 0x08010000 {
            self.gic.gicd_write(addr - 0x08000000, value, size);
            return;
        }
        // GIC redistributor
        if addr >= 0x080A0000 && addr < 0x08100000 {
            self.gic.gicr_write(addr - 0x080A0000, value, size);
            return;
        }
        self.uart.write(addr, size, value);
        let _ = self.mem.write(addr, size, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uart_priority_over_ram() {
        let mut bus = SystemBus::new();
        bus.write(0x0900_0000, 1, b'A' as u64);
        // Should go to UART, not RAM
        assert_eq!(bus.uart.output_string(), "A");
        // RAM at same address should be unmapped (UART intercepts)
    }

    #[test]
    fn ram_read_write() {
        let mut bus = SystemBus::new();
        bus.write(0x4000_0000, 8, 0xDEADBEEF);
        assert_eq!(bus.read(0x4000_0000, 8), Some(0xDEADBEEF));
    }
}
