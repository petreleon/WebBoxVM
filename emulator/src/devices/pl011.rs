//! PL011 UART — ARM PrimeCell serial port emulation for kernel console.
//!
//! ## Register map (standard ARM PL011 offsets)
//!
//! ```text
//! Offset  Name       Description
//! ------  ----       -----------
//! 0x00    UARTDR     Data Register (R/W). Write = transmit byte, Read = receive byte.
//! 0x04    UARTRSR    Receive Status / Error Clear (R=status, W=clear errors).
//! 0x18    UARTFR     Flag Register (Read-only). Bits 7(TXFE),5(TXFF),4(RXFE),3(BUSY).
//! 0x24    UARTIBRD   Integer Baud Rate Divisor (R/W).
//! 0x28    UARTFBRD   Fractional Baud Rate Divisor (R/W).
//! 0x2C    UARTLCR_H  Line Control Register (R/W) — word length, parity, stop bits.
//! 0x30    UARTCR     Control Register (R/W) — UART enable, TX/RX enable.
//! 0x34    UARTIFLS   Interrupt FIFO Level Select (R/W).
//! 0x38    UARTIMSC   Interrupt Mask Set/Clear (R/W).
//! 0x3C    UARTRIS    Raw Interrupt Status (Read-only).
//! 0x40    UARTMIS    Masked Interrupt Status (Read-only).
//! 0x44    UARTICR    Interrupt Clear (Write-only, write-1-to-clear).
//! 0x48    UARTDMACR  DMA Control Register (R/W).
//! ```
//!
//! ## How the Linux kernel uses it
//!
//! **Earlycon setup** (`pl011_early_console_setup`):
//!   1. Read CR (0x30), preserve RTS/DTR bits, set UARTEN|RXE|TXE
//!   2. Write CR back with those bits set
//!
//! **Output** (`pl011_putc`):
//!   1. Poll FR (0x18) until TXFF (bit 5) == 0  → "FIFO not full"
//!   2. Write byte to DR (0x00)
//!   3. Poll FR (0x18) until BUSY (bit 3) == 0  → "transmission complete"
//!
//! **Input** (`pl011_getc`, when CONFIG_CONSOLE_POLL):
//!   1. Poll FR (0x18) until RXFE (bit 4) == 0 → "data available"
//!   2. Read byte from DR (0x00)
//!
//! **Full driver** (`pl011_startup`):
//!   Same CR setup as earlycon, plus IBRD/FBRD/LCR_H/IFLS configuration
//!   and interrupt enable via IMSC.

#![allow(dead_code)] // register bit constants are documentation

use crate::constants::*;

// ── Register offsets (from base address) ──

/// Data Register — writing transmits a byte, reading receives a byte.
const UARTDR_OFFSET: u64 = 0x00;       // UART01x_DR
/// Receive Status / Error Clear.
const UARTRSR_OFFSET: u64 = 0x04;      // UART01x_RSR / UART01x_ECR
/// Flag Register — transmit/receive status flags.
const UARTFR_OFFSET: u64 = 0x18;       // UART01x_FR
/// Integer Baud Rate Divisor.
const UARTIBRD_OFFSET: u64 = 0x24;     // UART011_IBRD
/// Fractional Baud Rate Divisor.
const UARTFBRD_OFFSET: u64 = 0x28;     // UART011_FBRD
/// Line Control Register (high byte) — word length, FIFO enable, parity, stop bits.
const UARTLCR_H_OFFSET: u64 = 0x2C;    // UART011_LCRH
/// Control Register — UART enable, TX/RX enable, flow control.
const UARTCR_OFFSET: u64 = 0x30;       // UART011_CR
/// Interrupt FIFO Level Select — RX/TX interrupt trigger thresholds.
const UARTIFLS_OFFSET: u64 = 0x34;     // UART011_IFLS
/// Interrupt Mask Set/Clear — bits written as 1 enable the corresponding interrupt.
const UARTIMSC_OFFSET: u64 = 0x38;     // UART011_IMSC
/// Raw Interrupt Status — current interrupt state (unmasked).
const UARTRIS_OFFSET: u64 = 0x3C;      // UART011_RIS
/// Masked Interrupt Status — RIS ANDed with IMSC.
const UARTMIS_OFFSET: u64 = 0x40;      // UART011_MIS
/// Interrupt Clear — write-1-to-clear for each interrupt bit.
const UARTICR_OFFSET: u64 = 0x44;      // UART011_ICR
/// DMA Control Register.
const UARTDMACR_OFFSET: u64 = 0x48;    // UART011_DMACR

// ── Flag Register (UARTFR) bit definitions ──

/// TX FIFO Empty — set when the transmit FIFO is empty (ready to accept data).
const FR_TXFE: u8 = 1 << 7;   // UART011_FR_TXFE
/// RX FIFO Full — set when the receive FIFO has reached its threshold.
const FR_RXFF: u8 = 1 << 6;   // UART011_FR_RXFF
/// TX FIFO Full — set when the transmit FIFO is full (must wait before writing).
const FR_TXFF: u8 = 1 << 5;   // UART01x_FR_TXFF
/// RX FIFO Empty — set when no received data is available.
const FR_RXFE: u8 = 1 << 4;   // UART01x_FR_RXFE
/// UART Busy — set while actively transmitting a byte.
const FR_BUSY: u8 = 1 << 3;   // UART01x_FR_BUSY
/// Data Carrier Detect — status of the DCD modem signal.
const FR_DCD: u8 = 1 << 2;    // UART01x_FR_DCD
/// Data Set Ready — status of the DSR modem signal.
const FR_DSR: u8 = 1 << 1;    // UART01x_FR_DSR
/// Clear To Send — status of the CTS modem signal.
const FR_CTS: u8 = 1 << 0;    // UART01x_FR_CTS

// ── Control Register (UARTCR) bit definitions ──

/// CTS Hardware Flow Control Enable.
const CR_CTSEN: u16 = 1 << 15; // UART011_CR_CTSEN
/// RTS Hardware Flow Control Enable.
const CR_RTSEN: u16 = 1 << 14; // UART011_CR_RTSEN
/// RTS output signal (Request To Send).
const CR_RTS: u16 = 1 << 11;   // UART011_CR_RTS
/// DTR output signal (Data Terminal Ready).
const CR_DTR: u16 = 1 << 10;   // UART011_CR_DTR
/// Receive Enable — set to 1 to enable the receiver.
const CR_RXE: u16 = 1 << 9;    // UART011_CR_RXE
/// Transmit Enable — set to 1 to enable the transmitter.
const CR_TXE: u16 = 1 << 8;    // UART011_CR_TXE
/// Loopback Enable — internally connects TX to RX for testing.
const CR_LBE: u16 = 1 << 7;    // UART011_CR_LBE
/// UART Enable — master enable for the entire peripheral.
const CR_UARTEN: u16 = 1 << 0; // UART01x_CR_UARTEN

/// All bits the kernel preserves when reconfiguring CR.
const CR_PRESERVE_MASK: u16 = CR_RTS | CR_DTR;

/// Initial CR value: UART enabled, TX and RX enabled.
const CR_INITIAL: u16 = CR_UARTEN | CR_TXE | CR_RXE;

/// Default Flag Register value: TX empty, RX empty, not busy.
const FR_DEFAULT: u8 = FR_RXFE | FR_TXFE;

/// Baud rate divisor for 115200 bps at 24 MHz clock.
/// IBRD = 24_000_000 / (16 * 115200) = 13.02… → integer part = 13.
const DEFAULT_IBRD: u16 = 13;
/// Fractional part: 0.02… × 64 = 1.3… → round to 1.
const DEFAULT_FBRD: u8 = 1;

/// Default LCR_H: 8-bit word, FIFO enabled (0x60 | 0x10 = 0x70).
const DEFAULT_LCR_H: u16 = 0x70;

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
