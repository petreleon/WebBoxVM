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

    pub fn set_irq_pending_for_cpu(&mut self, core: usize, int_id: u32) {
        self.gic.set_pending_for_cpu(core, int_id);
    }

    pub fn clear_irq_pending(&mut self, int_id: u32) {
        self.clear_irq_pending_for_cpu(0, int_id);
    }

    pub fn clear_irq_pending_for_cpu(&mut self, core: usize, int_id: u32) {
        self.gic.clear_pending_for_cpu(core, int_id);
        if int_id == PL011_UART_IRQ_ID {
            self.mark_uart_rx_refresh_needed();
        }
    }

    pub fn acknowledge_irq_for_cpu(&mut self, core: usize, int_id: u32) -> bool {
        let acknowledged = self.gic.acknowledge_interrupt_for_cpu(core, int_id);
        if acknowledged && int_id == PL011_UART_IRQ_ID {
            self.mark_uart_rx_refresh_needed();
        }
        acknowledged
    }

    pub fn deactivate_irq_for_cpu(&mut self, core: usize, int_id: u32) -> bool {
        if int_id == PL011_UART_IRQ_ID {
            if self.uart.masked_rx_interrupt_pending() {
                self.gic.set_pending(int_id);
            } else {
                self.gic.clear_pending(int_id);
            }
        }
        let deactivated = self.gic.deactivate_interrupt_for_cpu(core, int_id);
        if deactivated && int_id == PL011_UART_IRQ_ID {
            self.mark_uart_rx_refresh_needed();
        }
        deactivated
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

    pub fn external_irq_poll_needed(&self) -> bool {
        self.external_irq_poll_needed_for_cpu(0)
    }

    pub fn external_irq_poll_needed_for_cpu(&self, core: usize) -> bool {
        self.uart_rx_refresh_needed || self.gic.has_pending_enabled_for_cpu(core)
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
