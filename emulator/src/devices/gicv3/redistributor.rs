use super::*;

impl Gicv3 {
    pub fn gicr_read(&self, offset: u64, _size: u8) -> Option<u64> {
        match offset {
            0x0000 => Some(self.rctlr),
            0x0004 => Some(self.iidr as u64),
            0x0008 => Some(self.rtyper),
            0x000C => Some(self.rtyper >> 32),
            0x0014 => Some(self.rwaker),
            0x001C => Some(self.rwaker >> 32),
            0xFFE8 => Some(GIC_PIDR2_ARCH_GICV3),
            _ => Some(0),
        }
    }

    pub fn gicr_write(&mut self, offset: u64, value: u64, _size: u8) {
        match offset {
            0x0000 => self.rctlr = value,
            0x0014 => self.rwaker = value,
            _ => {}
        }
    }
}
