//! PL011 UART register and bit definitions.

// ── Register offsets (from base address) ──

/// Data Register — writing transmits a byte, reading receives a byte.
pub(super) const UARTDR_OFFSET: u64 = 0x00;       // UART01x_DR
/// Receive Status / Error Clear.
pub(super) const UARTRSR_OFFSET: u64 = 0x04;      // UART01x_RSR / UART01x_ECR
/// Flag Register — transmit/receive status flags.
pub(super) const UARTFR_OFFSET: u64 = 0x18;       // UART01x_FR
/// Integer Baud Rate Divisor.
pub(super) const UARTIBRD_OFFSET: u64 = 0x24;     // UART011_IBRD
/// Fractional Baud Rate Divisor.
pub(super) const UARTFBRD_OFFSET: u64 = 0x28;     // UART011_FBRD
/// Line Control Register (high byte) — word length, FIFO enable, parity, stop bits.
pub(super) const UARTLCR_H_OFFSET: u64 = 0x2C;    // UART011_LCRH
/// Control Register — UART enable, TX/RX enable, flow control.
pub(super) const UARTCR_OFFSET: u64 = 0x30;       // UART011_CR
/// Interrupt FIFO Level Select — RX/TX interrupt trigger thresholds.
pub(super) const UARTIFLS_OFFSET: u64 = 0x34;     // UART011_IFLS
/// Interrupt Mask Set/Clear — bits written as 1 enable the corresponding interrupt.
pub(super) const UARTIMSC_OFFSET: u64 = 0x38;     // UART011_IMSC
/// Raw Interrupt Status — current interrupt state (unmasked).
pub(super) const UARTRIS_OFFSET: u64 = 0x3C;      // UART011_RIS
/// Masked Interrupt Status — RIS ANDed with IMSC.
pub(super) const UARTMIS_OFFSET: u64 = 0x40;      // UART011_MIS
/// Interrupt Clear — write-1-to-clear for each interrupt bit.
pub(super) const UARTICR_OFFSET: u64 = 0x44;      // UART011_ICR
/// DMA Control Register.
pub(super) const UARTDMACR_OFFSET: u64 = 0x48;    // UART011_DMACR

// ── Flag Register (UARTFR) bit definitions ──

/// TX FIFO Empty — set when the transmit FIFO is empty (ready to accept data).
pub(super) const FR_TXFE: u8 = 1 << 7;   // UART011_FR_TXFE
/// RX FIFO Full — set when the receive FIFO has reached its threshold.
pub(super) const FR_RXFF: u8 = 1 << 6;   // UART011_FR_RXFF
/// TX FIFO Full — set when the transmit FIFO is full (must wait before writing).
pub(super) const FR_TXFF: u8 = 1 << 5;   // UART01x_FR_TXFF
/// RX FIFO Empty — set when no received data is available.
pub(super) const FR_RXFE: u8 = 1 << 4;   // UART01x_FR_RXFE
/// UART Busy — set while actively transmitting a byte.
pub(super) const FR_BUSY: u8 = 1 << 3;   // UART01x_FR_BUSY
/// Data Carrier Detect — status of the DCD modem signal.
pub(super) const FR_DCD: u8 = 1 << 2;    // UART01x_FR_DCD
/// Data Set Ready — status of the DSR modem signal.
pub(super) const FR_DSR: u8 = 1 << 1;    // UART01x_FR_DSR
/// Clear To Send — status of the CTS modem signal.
pub(super) const FR_CTS: u8 = 1 << 0;    // UART01x_FR_CTS

// ── Control Register (UARTCR) bit definitions ──

/// CTS hardware flow control enable.
pub(super) const CR_CTSEN: u16 = 1 << 15; // UART011_CR_CTSEN
/// RTS hardware flow control enable.
pub(super) const CR_RTSEN: u16 = 1 << 14; // UART011_CR_RTSEN
/// Request To Send — output signal level.
pub(super) const CR_RTS: u16 = 1 << 11;   // UART011_CR_RTS
/// Data Terminal Ready — output signal level.
pub(super) const CR_DTR: u16 = 1 << 10;   // UART011_CR_DTR
/// Receive Enable — enables the receiver.
pub(super) const CR_RXE: u16 = 1 << 9;    // UART011_CR_RXE
/// Transmit Enable — enables the transmitter.
pub(super) const CR_TXE: u16 = 1 << 8;    // UART011_CR_TXE
/// Loopback Enable — internal loopback for testing.
pub(super) const CR_LBE: u16 = 1 << 7;    // UART011_CR_LBE
/// UART Enable — master enable for the UART.
pub(super) const CR_UARTEN: u16 = 1 << 0; // UART01x_CR_UARTEN

/// Bits in CR that should be preserved across writes (RTS, DTR).
pub(super) const CR_PRESERVE_MASK: u16 = CR_RTS | CR_DTR;

/// Initial CR value: UART enabled, TX and RX enabled.
pub(super) const CR_INITIAL: u16 = CR_UARTEN | CR_TXE | CR_RXE;

/// Default FR value at boot: RX FIFO empty, TX FIFO empty, not busy.
pub(super) const FR_DEFAULT: u8 = FR_RXFE | FR_TXFE;

/// Default integer baud-rate divisor (13 = 115200 @ UARTCLK=24 MHz).
pub(super) const DEFAULT_IBRD: u16 = 13;
/// Default fractional baud-rate divisor (1).
pub(super) const DEFAULT_FBRD: u8 = 1;

/// Default line-control setting: FIFOs enabled, 8-bit words, no parity, 1 stop bit.
pub(super) const DEFAULT_LCR_H: u16 = 0x70;
