use super::registers::*;
use super::*;

#[test]
fn earlycon_write_byte_through_flag_poll() {
    let mut uart = Pl011Uart::new();
    let dr = UART_BASE + UARTDR_OFFSET;
    let fr = UART_BASE + UARTFR_OFFSET;

    let flags = uart.read(fr, 4).unwrap() as u8;
    assert_eq!(flags & FR_TXFF, 0, "TX FIFO should not be full");

    uart.write(dr, 1, b'H' as u64);

    let flags = uart.read(fr, 4).unwrap() as u8;
    assert_eq!(flags & FR_BUSY, 0, "UART should not be busy");

    assert_eq!(uart.output_string(), "H");
}

#[test]
fn earlycon_setup_reads_and_writes_cr() {
    let mut uart = Pl011Uart::new();
    let cr_addr = UART_BASE + UARTCR_OFFSET;

    let cr = uart.read(cr_addr, 4).unwrap() as u16;
    assert_eq!(
        cr & CR_UARTEN,
        CR_UARTEN,
        "UART should be enabled initially"
    );

    let new_cr = (cr & CR_PRESERVE_MASK) | CR_UARTEN | CR_TXE | CR_RXE;
    uart.write(cr_addr, 4, new_cr as u64);

    let saved = uart.read(cr_addr, 4).unwrap() as u16;
    assert_eq!(saved & CR_UARTEN, CR_UARTEN, "UART should remain enabled");
    assert_eq!(saved & CR_TXE, CR_TXE, "TX should be enabled");
    assert_eq!(saved & CR_RXE, CR_RXE, "RX should be enabled");
}

#[test]
fn full_startup_writes_cr_then_enables_interrupts() {
    let mut uart = Pl011Uart::new();
    let cr_addr = UART_BASE + UARTCR_OFFSET;
    let imsc_addr = UART_BASE + UARTIMSC_OFFSET;
    let ifls_addr = UART_BASE + UARTIFLS_OFFSET;

    uart.write(ifls_addr, 4, 0x12);
    assert_eq!(uart.read(ifls_addr, 4).unwrap(), 0x12);

    let cr = uart.read(cr_addr, 4).unwrap() as u16;
    let new_cr = (cr & CR_PRESERVE_MASK) | CR_UARTEN | CR_TXE | CR_RXE;
    uart.write(cr_addr, 4, new_cr as u64);

    uart.write(imsc_addr, 4, 0x50);

    assert_eq!(uart.read(cr_addr, 4).unwrap() as u16 & CR_TXE, CR_TXE);
    assert_ne!(uart.read(imsc_addr, 4).unwrap(), 0);
}

#[test]
fn getc_when_no_input_available() {
    let mut uart = Pl011Uart::new();
    let fr = uart.read(UART_BASE + UARTFR_OFFSET, 4).unwrap() as u8;

    assert_ne!(fr & FR_RXFE, 0, "RX FIFO should be empty");
}

#[test]
fn getc_reads_queued_input() {
    let mut uart = Pl011Uart::new();
    uart.feed_input_byte(b'X');

    let fr = uart.read(UART_BASE + UARTFR_OFFSET, 4).unwrap() as u8;
    assert_eq!(
        fr & FR_RXFE,
        0,
        "RX FIFO should NOT be empty when data queued"
    );

    let ch = uart.read(UART_BASE + UARTDR_OFFSET, 4).unwrap();
    assert_eq!(ch as u8, b'X');

    let fr = uart.read(UART_BASE + UARTFR_OFFSET, 4).unwrap() as u8;
    assert_ne!(fr & FR_RXFE, 0, "RX FIFO should be empty after draining");
}

#[test]
fn baud_rate_registers_have_sensible_defaults() {
    let mut uart = Pl011Uart::new();
    assert!(uart.read(UART_BASE + UARTIBRD_OFFSET, 4).is_some());
    assert!(uart.read(UART_BASE + UARTFBRD_OFFSET, 4).is_some());
}

#[test]
fn primecell_id_registers_identify_pl011() {
    let mut uart = Pl011Uart::new();
    assert_eq!(uart.read(UART_BASE + UARTPERIPHID0_OFFSET, 4), Some(0x11));
    assert_eq!(uart.read(UART_BASE + UARTPERIPHID1_OFFSET, 4), Some(0x10));
    assert_eq!(uart.read(UART_BASE + UARTPERIPHID2_OFFSET, 4), Some(0x14));
    assert_eq!(uart.read(UART_BASE + UARTPERIPHID3_OFFSET, 4), Some(0x00));
    assert_eq!(uart.read(UART_BASE + UARTPCELLID0_OFFSET, 4), Some(0x0D));
    assert_eq!(uart.read(UART_BASE + UARTPCELLID1_OFFSET, 4), Some(0xF0));
    assert_eq!(uart.read(UART_BASE + UARTPCELLID2_OFFSET, 4), Some(0x05));
    assert_eq!(uart.read(UART_BASE + UARTPCELLID3_OFFSET, 4), Some(0xB1));
}

#[test]
fn interrupt_registers_read_zero_when_no_pending_irqs() {
    let mut uart = Pl011Uart::new();
    assert_eq!(uart.read(UART_BASE + UARTRIS_OFFSET, 4).unwrap(), 0);
    assert_eq!(uart.read(UART_BASE + UARTMIS_OFFSET, 4).unwrap(), 0);
}

#[test]
fn rx_input_sets_masked_interrupt_status() {
    let mut uart = Pl011Uart::new();
    uart.write(UART_BASE + UARTIMSC_OFFSET, 4, (INT_RX | INT_RT) as u64);
    uart.feed_input("x");

    assert_ne!(
        uart.read(UART_BASE + UARTRIS_OFFSET, 4).unwrap() as u16 & INT_RX,
        0
    );
    assert_ne!(
        uart.read(UART_BASE + UARTMIS_OFFSET, 4).unwrap() as u16 & INT_RX,
        0
    );

    assert_eq!(uart.read(UART_BASE + UARTDR_OFFSET, 4).unwrap() as u8, b'x');
    assert_eq!(uart.read(UART_BASE + UARTRIS_OFFSET, 4).unwrap(), 0);
}
