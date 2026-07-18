//! PL011 UART — ARM PrimeCell serial port emulation.

use crate::constants::*;
use std::collections::VecDeque;

mod io;
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

    pub(crate) fn cold_reset(&mut self) {
        let output = std::mem::take(&mut self.output);
        *self = Self::new();
        self.output = output;
    }

    pub(in crate::devices::pl011) fn raw_interrupt_status(&self) -> u16 {
        if self.input.is_empty() {
            0
        } else {
            INT_RX | INT_RT
        }
    }

    pub fn masked_rx_interrupt_pending(&self) -> bool {
        self.raw_interrupt_status() & self.imsc & (INT_RX | INT_RT) != 0
    }
}

pub(in crate::devices::pl011) fn is_uart_addr(addr: u64) -> bool {
    addr >= UART_BASE && addr < UART_END
}

#[cfg(test)]
mod tests;
