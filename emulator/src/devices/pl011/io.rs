use super::*;

impl Pl011Uart {
    /// Handle MMIO read at any register offset within the UART window.
    pub fn read(&mut self, addr: u64, _size: u8) -> Option<u64> {
        if !is_uart_addr(addr) {
            return None;
        }
        match addr - UART_BASE {
            UARTDR_OFFSET => Some(self.input.pop_front().unwrap_or(0) as u64),
            UARTRSR_OFFSET => Some(0),
            UARTFR_OFFSET => Some(self.flags() as u64),
            UARTIBRD_OFFSET => Some(self.ibrd as u64),
            UARTFBRD_OFFSET => Some(self.fbrd as u64),
            UARTLCR_H_OFFSET => Some(self.lcr_h as u64),
            UARTCR_OFFSET => Some(self.cr as u64),
            UARTIFLS_OFFSET => Some(self.ifls as u64),
            UARTIMSC_OFFSET => Some(self.imsc as u64),
            UARTRIS_OFFSET => Some(self.raw_interrupt_status() as u64),
            UARTMIS_OFFSET => Some((self.raw_interrupt_status() & self.imsc) as u64),
            UARTDMACR_OFFSET => Some(0),
            UARTPERIPHID0_OFFSET => Some(0x11),
            UARTPERIPHID1_OFFSET => Some(0x10),
            UARTPERIPHID2_OFFSET => Some(0x14),
            UARTPERIPHID3_OFFSET => Some(0x00),
            UARTPCELLID0_OFFSET => Some(0x0D),
            UARTPCELLID1_OFFSET => Some(0xF0),
            UARTPCELLID2_OFFSET => Some(0x05),
            UARTPCELLID3_OFFSET => Some(0xB1),
            _ => Some(0),
        }
    }

    /// Handle MMIO write at any register offset within the UART window.
    pub fn write(&mut self, addr: u64, _size: u8, value: u64) {
        if !is_uart_addr(addr) {
            return;
        }
        match addr - UART_BASE {
            UARTDR_OFFSET => self.output.push(value as u8),
            UARTRSR_OFFSET | UARTICR_OFFSET | UARTDMACR_OFFSET => {}
            UARTCR_OFFSET => self.cr = (value as u16) & writable_control_bits(),
            UARTIBRD_OFFSET => self.ibrd = value as u16,
            UARTFBRD_OFFSET => self.fbrd = (value & 0x3F) as u8,
            UARTLCR_H_OFFSET => self.lcr_h = value as u16,
            UARTIFLS_OFFSET => self.ifls = value as u16,
            UARTIMSC_OFFSET => self.imsc = value as u16,
            _ => {}
        }
    }

    fn flags(&self) -> u8 {
        let mut fr = FR_DEFAULT;
        if self.input.is_empty() {
            fr |= FR_RXFE;
        } else {
            fr &= !FR_RXFE;
            if self.input.len() >= 16 {
                fr |= FR_RXFF;
            }
        }
        fr
    }
}

fn writable_control_bits() -> u16 {
    CR_UARTEN | CR_TXE | CR_RXE | CR_LBE | CR_RTS | CR_DTR | CR_RTSEN | CR_CTSEN
}
