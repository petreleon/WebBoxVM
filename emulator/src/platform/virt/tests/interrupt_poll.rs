use crate::constants::{PL011_UART_IRQ_ID, UART_BASE, UART_IMSC_OFFSET};
use crate::platform::virt::SystemBus;

#[test]
fn external_irq_poll_needed_tracks_enabled_pending_and_uart_refresh() {
    let mut bus = SystemBus::new();

    assert!(!bus.external_irq_poll_needed());

    bus.set_irq_pending(PL011_UART_IRQ_ID);
    assert!(!bus.external_irq_poll_needed());

    bus.gic.enable_interrupt(PL011_UART_IRQ_ID);
    assert!(bus.external_irq_poll_needed());

    bus.clear_irq_pending(PL011_UART_IRQ_ID);
    assert!(bus.external_irq_poll_needed());

    bus.refresh_interrupts();
    assert!(!bus.external_irq_poll_needed());

    bus.write(UART_BASE + UART_IMSC_OFFSET, 4, 0x50);
    bus.feed_uart_input("x");
    bus.clear_irq_pending(PL011_UART_IRQ_ID);
    assert!(bus.external_irq_poll_needed());

    bus.refresh_interrupts();
    assert!(bus.gic.is_pending(PL011_UART_IRQ_ID));
    assert!(bus.external_irq_poll_needed());
}
