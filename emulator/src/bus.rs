//! MMIO dispatch table: routes physical addresses to device handlers.

use crate::devices::pl011::Pl011Uart;
use crate::memory::PhysicalMemory;

pub struct SystemBus {
    pub mem: PhysicalMemory,
    pub uart: Pl011Uart,
}

impl SystemBus {
    pub fn new() -> Self {
        Self {
            mem: PhysicalMemory::new(),
            uart: Pl011Uart::new(),
        }
    }

    pub fn read(&self, addr: u64, size: u8) -> Option<u64> {
        self.uart.read(addr, size)
            .or_else(|| self.mem.read(addr, size))
    }

    pub fn write(&mut self, addr: u64, size: u8, value: u64) {
        self.uart.write(addr, size, value);
        let _ = self.mem.write(addr, size, value);
        if addr <= 0x41fdf70d && addr + size as u64 > 0x41fdf70d {
            eprintln!("BUS WRITE: addr=0x{:016x} size={} value=0x{:016x}", addr, size, value);
        }
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
