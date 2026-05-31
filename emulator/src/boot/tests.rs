use super::*;

#[test]
fn feeding_uart_input_queues_rx_and_injects_irq() {
    let mut ctx = BootContext::new(&[0u8; 64], 1).unwrap();

    ctx.feed_uart_input("ls\r");

    assert_ne!(
        ctx.machine.bus.gic.pending[(PL011_UART_IRQ_ID / 32) as usize]
            & (1 << (PL011_UART_IRQ_ID % 32)),
        0
    );
    assert_eq!(
        ctx.machine
            .bus
            .read(UART_BASE + UART_RIS_OFFSET, 4)
            .unwrap() as u16
            & (1 << 4),
        1 << 4
    );
    assert_eq!(
        ctx.machine.bus.read(UART_BASE + UART_DR_OFFSET, 4).unwrap() as u8,
        b'l'
    );
}
