use super::*;

impl SystemBus {
    pub fn feed_uart_input(&mut self, input: &str) {
        self.feed_uart_bytes(input.as_bytes());
    }

    pub fn feed_uart_bytes(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.uart.feed_input_bytes(bytes);
        self.set_irq_pending(PL011_UART_IRQ_ID);
    }

    pub fn set_irq_pending(&mut self, int_id: u32) {
        self.gic.set_pending(int_id);
    }

    pub fn clear_irq_pending(&mut self, int_id: u32) {
        self.gic.clear_pending(int_id);
        if int_id == PL011_UART_IRQ_ID {
            self.mark_uart_rx_refresh_needed();
        }
    }

    pub fn refresh_interrupts(&mut self) {
        if !self.uart_rx_refresh_needed {
            return;
        }
        self.uart_rx_refresh_needed = false;
        if self.uart.masked_rx_interrupt_pending() {
            self.gic.set_pending(PL011_UART_IRQ_ID);
        }
    }

    pub(super) fn mark_uart_rx_refresh_needed(&mut self) {
        self.uart_rx_refresh_needed = true;
    }
}

pub(super) fn gicd_clear_pending_touches_uart(addr: u64, value: u64) -> bool {
    let offset = addr - GICD_BASE;
    if !(0x0280..0x0300).contains(&offset) {
        return false;
    }
    let idx = ((offset - 0x0280) / 4) as u32;
    idx == PL011_UART_IRQ_ID / 32 && (value as u32 & (1 << (PL011_UART_IRQ_ID % 32))) != 0
}
