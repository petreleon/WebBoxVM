//! PL011 UART — ARM PrimeCell serial port emulation.

use crate::constants::*;
use std::collections::VecDeque;

mod registers;
use registers::*;

pub struct Pl011Uart {
    pub output: Vec<u8>,
    input: VecDeque<u8>, // bytes queued for the guest to read
    cr: u16,             // Control Register
    lcr_h: u16,          // Line Control Register (high)
    ibrd: u16,           // Integer Baud Rate Divisor
    fbrd: u8,            // Fractional Baud Rate Divisor
    ifls: u16,           // Interrupt FIFO Level Select
    imsc: u16,           // Interrupt Mask Set/Clear
}

impl Pl011Uart {
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
            input: VecDeque::new(),
            cr: CR_INITIAL, // virtual UART starts "enabled"
            lcr_h: DEFAULT_LCR_H,
            ibrd: DEFAULT_IBRD,
            fbrd: DEFAULT_FBRD,
            ifls: 0,
            imsc: 0,
        }
    }

    /// Handle MMIO read at any register offset within the UART window.
    ///
    /// Returns `None` if the address is outside the UART range.
    pub fn read(&mut self, addr: u64, _size: u8) -> Option<u64> {
        if !is_uart_addr(addr) {
            return None;
        }
        let offset = addr - UART_BASE;
        match offset {
            UARTDR_OFFSET => {
                // Reading from DR consumes the first queued input byte.
                Some(self.input.pop_front().unwrap_or(0) as u64)
            }
            UARTRSR_OFFSET => {
                // No errors in a virtual UART — always return 0.
                Some(0)
            }
            UARTFR_OFFSET => {
                let mut fr = FR_DEFAULT;
                // TX FIFO is never full in our virtual UART
                // RX data available only if we have queued input
                if self.input.is_empty() {
                    fr |= FR_RXFE;
                } else {
                    fr &= !FR_RXFE;
                    if self.input.len() >= 16 {
                        fr |= FR_RXFF;
                    }
                }
                Some(fr as u64)
            }
            UARTIBRD_OFFSET => Some(self.ibrd as u64),
            UARTFBRD_OFFSET => Some(self.fbrd as u64),
            UARTLCR_H_OFFSET => Some(self.lcr_h as u64),
            UARTCR_OFFSET => Some(self.cr as u64),
            UARTIFLS_OFFSET => Some(self.ifls as u64),
            UARTIMSC_OFFSET => Some(self.imsc as u64),
            UARTRIS_OFFSET => Some(self.raw_interrupt_status() as u64),
            UARTMIS_OFFSET => Some((self.raw_interrupt_status() & self.imsc) as u64),
            UARTDMACR_OFFSET => Some(0),
            _ => Some(0), // Reserved/gap registers return 0
        }
    }

    /// Handle MMIO write at any register offset within the UART window.
    pub fn write(&mut self, addr: u64, _size: u8, value: u64) {
        if !is_uart_addr(addr) {
            return;
        }
        let offset = addr - UART_BASE;
        match offset {
            UARTDR_OFFSET => {
                // Writing to DR transmits a byte (captured in output queue).
                self.output.push(value as u8);
            }
            UARTRSR_OFFSET => {
                // Writing to ECR clears error flags — no-op in virtual UART.
            }
            UARTCR_OFFSET => {
                // Store only the writable bits; preserve reserved bits.
                self.cr = (value as u16)
                    & (CR_UARTEN
                        | CR_TXE
                        | CR_RXE
                        | CR_LBE
                        | CR_RTS
                        | CR_DTR
                        | CR_RTSEN
                        | CR_CTSEN);
            }
            UARTIBRD_OFFSET => {
                self.ibrd = value as u16;
            }
            UARTFBRD_OFFSET => {
                self.fbrd = (value & 0x3F) as u8; // FBRD is 6-bit
            }
            UARTLCR_H_OFFSET => {
                self.lcr_h = value as u16;
            }
            UARTIFLS_OFFSET => {
                self.ifls = value as u16;
            }
            UARTIMSC_OFFSET => {
                // Kernel enables RX and RX-timeout interrupts during init.
                // This is harmless — we have no real IRQ delivery anyway.
                self.imsc = value as u16;
            }
            UARTICR_OFFSET => {
                // Write-1-to-clear interrupts. The kernel clears pending RX
                // and error interrupts after initialization. No-op for us.
            }
            UARTDMACR_OFFSET => {
                // DMA control — ignored in our simple emulation.
            }
            _ => {
                // Ignore writes to reserved or unimplemented registers.
            }
        }
    }

    /// Feed a byte into the UART's receive path (for guest input simulation).
    pub fn feed_input_byte(&mut self, byte: u8) {
        self.input.push_back(byte);
    }

    /// Feed bytes into the UART's receive path.
    pub fn feed_input_bytes(&mut self, bytes: &[u8]) {
        self.input.extend(bytes.iter().copied());
    }

    /// Feed a string into the UART's receive path.
    pub fn feed_input(&mut self, s: &str) {
        self.feed_input_bytes(s.as_bytes());
    }

    /// Return all accumulated output as a UTF-8 string.
    pub fn output_string(&self) -> String {
        String::from_utf8_lossy(&self.output).to_string()
    }

    fn raw_interrupt_status(&self) -> u16 {
        if self.input.is_empty() {
            0
        } else {
            INT_RX | INT_RT
        }
    }
}

/// Returns true if `addr` falls inside the UART MMIO window.
fn is_uart_addr(addr: u64) -> bool {
    addr >= UART_BASE && addr < UART_END
}

#[cfg(test)]
mod tests;
