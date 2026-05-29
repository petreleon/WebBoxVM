//! PL011 UART — minimal implementation for kernel debug console output.
//!
//! The PL011 is a standard ARM PrimeCell UART.  We only implement two registers:
//!   - DR   (Data Register, offset 0x00): write a byte to transmit it
//!   - FR   (Flag Register, offset 0x18): read to check transmit readiness

use crate::constants::*;

/// UART Data Register — writing a byte here sends it over the serial line.
const UARTDR: u64 = UART_BASE + 0x00;

/// UART Flag Register.
/// Bit 7 (TXFE) = 1 means the transmit FIFO is empty (ready to send).
/// Bit 4 (RXFE) = 1 means the receive FIFO is empty (no incoming data).
const UARTFR: u64 = UART_BASE + 0x18;

/// UARTFR value: TX empty (bit 7), RX empty (bit 4) → 0b1001_0000 = 0x90.
const UARTFR_TXFE_RXFE: u64 = 0x90;

pub struct Pl011Uart {
    pub output: Vec<u8>,
}

impl Pl011Uart {
    pub fn new() -> Self {
        Self { output: Vec::new() }
    }

    /// Handle MMIO read. Returns the register value, or 0 for unmapped fields.
    pub fn read(&self, addr: u64, _size: u8) -> Option<u64> {
        if addr >= UART_BASE && addr < UART_END {
            match addr {
                UARTDR => Some(0),                  // always empty on read
                UARTFR => Some(UARTFR_TXFE_RXFE),  // always ready to transmit
                _ => Some(0),                       // other registers: no errors
            }
        } else {
            None
        }
    }

    /// Handle MMIO write. Only the DR register is actually written.
    pub fn write(&mut self, addr: u64, _size: u8, value: u64) {
        if addr == UARTDR {
            self.output.push(value as u8);
        }
    }

    /// Return all accumulated output as a UTF-8 string.
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
    fn flag_register_ready() {
        let uart = Pl011Uart::new();
        assert_eq!(uart.read(UARTFR, 4), Some(UARTFR_TXFE_RXFE));
    }
}
