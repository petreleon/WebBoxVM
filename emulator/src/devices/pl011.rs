//! PL011 UART minimal implementation for kernel debug output.

const UARTDR: u64 = 0x0900_0000; // Data register (R/W)
const UARTFR: u64 = 0x0900_0018; // Flag register (R)

pub struct Pl011Uart {
    pub output: Vec<u8>,
}

impl Pl011Uart {
    pub fn new() -> Self {
        Self { output: Vec::new() }
    }

    /// Handle MMIO read. Returns value or 0 for all registers in range.
    pub fn read(&self, addr: u64, _size: u8) -> Option<u64> {
        if addr >= 0x09000000 && addr < 0x09001000 {
            match addr {
                UARTDR => Some(0),         // No input
                UARTFR => Some(0x90),      // TXFF=0, RXFE=1 → ready to transmit
                _ => Some(0),              // All other registers return 0 (no errors)
            }
        } else {
            None
        }
    }

    /// Handle MMIO write.
    pub fn write(&mut self, addr: u64, _size: u8, value: u64) {
        if addr == UARTDR {
            self.output.push(value as u8);
        }
    }

    /// Return accumulated output as string (valid UTF-8 prefix).
    pub fn output_string(&self) -> String {
        String::from_utf8_lossy(&self.output).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_char() {
        let mut uart = Pl011Uart::new();
        uart.write(UARTDR, 1, b'H' as u64);
        uart.write(UARTDR, 1, b'i' as u64);
        assert_eq!(uart.output_string(), "Hi");
    }

    #[test]
    fn flag_register() {
        let uart = Pl011Uart::new();
        assert_eq!(uart.read(UARTFR, 4), Some(0x90));
    }
}
