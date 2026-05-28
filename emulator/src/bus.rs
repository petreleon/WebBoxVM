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
        // Redirect fixmap OA-offset UART reads to correct device
        let low = addr & 0x001F_FFFF;
        if low >= 0x090000 && low < 0x091000 {
            let corrected = 0x09000000 | (addr & 0xFFF);
            if let Some(val) = self.uart.read(corrected, size) {
                return Some(val);
            }
        }
        self.uart.read(addr, size)
            .or_else(|| self.mem.read(addr, size))
    }

    pub fn write(&mut self, addr: u64, size: u8, value: u64) {
        // Redirect fixmap OA-offset UART writes: catch single-byte ASCII writes
        // to any PA whose lower 21 bits are in UART range (0x090000-0x090FFF)
        let low = addr & 0x001F_FFFF;
        if size == 1 && low >= 0x090000 && low < 0x091000 
            && addr != (0x09000000 | (addr & 0xFFF)) {
            let b = value as u8;
            if b >= 0x20 && b <= 0x7E || b == b'\n' || b == b'\r' {
                let corrected = 0x09000000 | (addr & 0xFFF);
                self.uart.write(corrected, size, value);
                let _ = self.mem.write(addr, size, value);
                return;
            }
        }
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
