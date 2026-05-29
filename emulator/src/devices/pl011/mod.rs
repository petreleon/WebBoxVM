//! PL011 UART — ARM PrimeCell serial port emulation.

use crate::constants::*;

mod registers;
use registers::*;

pub struct Pl011Uart {
    pub output: Vec<u8>,
    input: Vec<u8>,       // bytes queued for the guest to read
    cr: u16,              // Control Register
    lcr_h: u16,           // Line Control Register (high)
    ibrd: u16,            // Integer Baud Rate Divisor
    fbrd: u8,             // Fractional Baud Rate Divisor
    ifls: u16,            // Interrupt FIFO Level Select
    imsc: u16,            // Interrupt Mask Set/Clear
}

impl Pl011Uart {
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
            input: Vec::new(),
            cr: CR_INITIAL,         // virtual UART starts "enabled"
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
                if self.input.is_empty() {
                    Some(0)
                } else {
                    Some(self.input.remove(0) as u64)
                }
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
                    fr |= FR_RXFF; // signal "data available"
                }
                Some(fr as u64)
            }
            UARTIBRD_OFFSET => Some(self.ibrd as u64),
            UARTFBRD_OFFSET => Some(self.fbrd as u64),
            UARTLCR_H_OFFSET => Some(self.lcr_h as u64),
            UARTCR_OFFSET => Some(self.cr as u64),
            UARTIFLS_OFFSET => Some(self.ifls as u64),
            UARTIMSC_OFFSET => Some(self.imsc as u64),
            UARTRIS_OFFSET => {
                // No real interrupts; TX always "ready" (bit 5 = TXIS)
                Some(0) // No pending interrupts
            }
            UARTMIS_OFFSET => {
                // Masked = RIS & IMSC. With no interrupts, always 0.
                Some(0)
            }
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
                self.cr = (value as u16) & (CR_UARTEN | CR_TXE | CR_RXE
                    | CR_LBE | CR_RTS | CR_DTR | CR_RTSEN | CR_CTSEN);
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
        self.input.push(byte);
    }

    /// Feed a string into the UART's receive path.
    pub fn feed_input(&mut self, s: &str) {
        self.input.extend_from_slice(s.as_bytes());
    }

    /// Return all accumulated output as a UTF-8 string.
    pub fn output_string(&self) -> String {
        String::from_utf8_lossy(&self.output).to_string()
    }
}

/// Returns true if `addr` falls inside the UART MMIO window.
fn is_uart_addr(addr: u64) -> bool {
    addr >= UART_BASE && addr < UART_END
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Earlycon: pl011_putc() output path ──

    #[test]
    fn earlycon_write_byte_through_flag_poll() {
        let mut uart = Pl011Uart::new();
        let dr = UART_BASE + UARTDR_OFFSET;
        let fr = UART_BASE + UARTFR_OFFSET;

        // 1. Kernel polls FR until TXFF (bit 5) == 0
        let flags = uart.read(fr, 4).unwrap() as u8;
        assert_eq!(flags & FR_TXFF, 0, "TX FIFO should not be full");

        // 2. Kernel writes byte 'H' to DR
        uart.write(dr, 1, b'H' as u64);

        // 3. Kernel polls FR until BUSY (bit 3) == 0
        let flags = uart.read(fr, 4).unwrap() as u8;
        assert_eq!(flags & FR_BUSY, 0, "UART should not be busy");

        assert_eq!(uart.output_string(), "H");
    }

    #[test]
    fn earlycon_setup_reads_and_writes_cr() {
        let mut uart = Pl011Uart::new();
        let cr_addr = UART_BASE + UARTCR_OFFSET;

        // Kernel's pl011_early_console_setup():
        //   cr = readl(base + UART011_CR);
        //   cr &= RTS | DTR;
        //   cr |= UARTEN | RXE | TXE;
        //   writel(cr, base + UART011_CR);
        let cr = uart.read(cr_addr, 4).unwrap() as u16;
        assert_eq!(cr & CR_UARTEN, CR_UARTEN, "UART should be enabled initially");

        // Kernel writes back: enable RX, TX, and preserve RTS/DTR
        let new_cr = (cr & CR_PRESERVE_MASK) | CR_UARTEN | CR_TXE | CR_RXE;
        uart.write(cr_addr, 4, new_cr as u64);

        let saved = uart.read(cr_addr, 4).unwrap() as u16;
        assert_eq!(saved & CR_UARTEN, CR_UARTEN, "UART should remain enabled");
        assert_eq!(saved & CR_TXE, CR_TXE, "TX should be enabled");
        assert_eq!(saved & CR_RXE, CR_RXE, "RX should be enabled");
    }

    // ── Full driver: pl011_startup() ──

    #[test]
    fn full_startup_writes_cr_then_enables_interrupts() {
        let mut uart = Pl011Uart::new();
        let cr_addr = UART_BASE + UARTCR_OFFSET;
        let imsc_addr = UART_BASE + UARTIMSC_OFFSET;

        // Kernel writes IFLS (FIFO level select) — common value 0x12
        let ifls_addr = UART_BASE + UARTIFLS_OFFSET;
        uart.write(ifls_addr, 4, 0x12);
        assert_eq!(uart.read(ifls_addr, 4).unwrap(), 0x12);

        // Kernel enables UART: CR |= UARTEN | RXE | TXE
        let cr = uart.read(cr_addr, 4).unwrap() as u16;
        let new_cr = (cr & CR_PRESERVE_MASK) | CR_UARTEN | CR_TXE | CR_RXE;
        uart.write(cr_addr, 4, new_cr as u64);

        // Kernel enables interrupts: IMSC = RTIM | RXIM
        uart.write(imsc_addr, 4, 0x50); // RTIM(0x40) | RXIM(0x10)

        // Verify
        assert_eq!(uart.read(cr_addr, 4).unwrap() as u16 & CR_TXE, CR_TXE);
        assert_ne!(uart.read(imsc_addr, 4).unwrap(), 0);
    }

    // ── Input path: pl011_getc() ──

    #[test]
    fn getc_when_no_input_available() {
        let mut uart = Pl011Uart::new();
        let fr = uart.read(UART_BASE + UARTFR_OFFSET, 4).unwrap() as u8;

        // RXFE (bit 4) should be set when no input is queued
        assert_ne!(fr & FR_RXFE, 0, "RX FIFO should be empty");
    }

    #[test]
    fn getc_reads_queued_input() {
        let mut uart = Pl011Uart::new();
        uart.feed_input_byte(b'X');

        // Kernel's pl011_getc():
        //   1. Check FR.RXFE — should NOT be set (data available)
        let fr = uart.read(UART_BASE + UARTFR_OFFSET, 4).unwrap() as u8;
        assert_eq!(fr & FR_RXFE, 0, "RX FIFO should NOT be empty when data queued");

        //   2. Read DR to get the byte
        let ch = uart.read(UART_BASE + UARTDR_OFFSET, 4).unwrap();
        assert_eq!(ch as u8, b'X');

        // After reading, RXFE should be set again (no more data)
        let fr = uart.read(UART_BASE + UARTFR_OFFSET, 4).unwrap() as u8;
        assert_ne!(fr & FR_RXFE, 0, "RX FIFO should be empty after draining");
    }

    // ── Baud rate divisor registers ──

    #[test]
    fn baud_rate_registers_have_sensible_defaults() {
        let mut uart = Pl011Uart::new();
        // Kernel reads IBRD/FBRD during full driver init but doesn't
        // rely on specific values for a virtual UART.
        assert!(uart.read(UART_BASE + UARTIBRD_OFFSET, 4).is_some());
        assert!(uart.read(UART_BASE + UARTFBRD_OFFSET, 4).is_some());
    }

    #[test]
    fn interrupt_registers_read_zero_when_no_pending_irqs() {
        let mut uart = Pl011Uart::new();
        assert_eq!(uart.read(UART_BASE + UARTRIS_OFFSET, 4).unwrap(), 0);
        assert_eq!(uart.read(UART_BASE + UARTMIS_OFFSET, 4).unwrap(), 0);
    }
}
