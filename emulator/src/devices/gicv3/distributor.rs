use super::*;

impl Gicv3 {
    pub fn gicd_read(&self, offset: u64, size: u8) -> Option<u64> {
        match offset {
            0x0000 => Some(self.ctld),
            0x0004 => Some(self.typer),
            0x0008 => Some(self.iidr as u64),
            0xFFE8 => Some(GIC_PIDR2_ARCH_GICV3),
            o if gicd_in_range(o, 0x0080, 0x0100) => {
                Some(read_bitmap_word(&self.group, gicd_word_index(o, 0x0080)))
            }
            o if gicd_in_range(o, 0x0100, 0x0180) => {
                Some(self.enable_word(gicd_word_index(o, 0x0100)) as u64)
            }
            o if gicd_in_range(o, 0x0180, 0x0200) => {
                Some(self.enable_word(gicd_word_index(o, 0x0180)) as u64)
            }
            o if gicd_in_range(o, 0x0200, 0x0280) => {
                Some(self.pending_word(gicd_word_index(o, 0x0200)) as u64)
            }
            o if gicd_in_range(o, 0x0280, 0x0300) => {
                Some(self.pending_word(gicd_word_index(o, 0x0280)) as u64)
            }
            o if gicd_in_range(o, 0x0400, 0x0800) => self.read_priority(o, size),
            o if gicd_in_range(o, 0x6000, 0x8000) => Some(self.read_irouter_mmio(o, size)),
            _ => Some(0),
        }
    }

    pub fn gicd_write(&mut self, offset: u64, value: u64, size: u8) {
        match offset {
            0x0000 => self.ctld = value,
            o if gicd_in_range(o, 0x0080, 0x0100) => {
                let idx = gicd_word_index(o, 0x0080);
                if idx < INT_WORDS {
                    self.group[idx] = value as u32;
                }
            }
            o if gicd_in_range(o, 0x0100, 0x0180) => {
                self.set_bitmap_word(o, 0x0100, value as u32);
            }
            o if gicd_in_range(o, 0x0180, 0x0200) => {
                self.clear_bitmap_word(o, 0x0180, value as u32);
            }
            o if gicd_in_range(o, 0x0200, 0x0280) => {
                let idx = gicd_word_index(o, 0x0200);
                self.set_pending_word_bits(idx, value as u32);
            }
            o if gicd_in_range(o, 0x0280, 0x0300) => {
                let idx = gicd_word_index(o, 0x0280);
                self.clear_pending_word_bits(idx, value as u32);
            }
            o if gicd_in_range(o, 0x0400, 0x0800) => self.write_priority(o, value, size),
            o if gicd_in_range(o, 0x6000, 0x8000) => {
                self.write_irouter_mmio(o, value, size);
            }
            _ => {}
        }
    }

    fn read_priority(&self, offset: u64, size: u8) -> Option<u64> {
        let idx = (offset - 0x0400) as usize;
        if idx >= MAX_INTERRUPTS {
            return Some(0);
        }
        let mut value = 0u64;
        for i in 0..8.min(size as usize) {
            if idx + i < MAX_INTERRUPTS {
                value |= (self.priority[idx + i] as u64) << (i * 8);
            }
        }
        Some(value)
    }

    fn write_priority(&mut self, offset: u64, value: u64, size: u8) {
        let idx = (offset - 0x0400) as usize;
        for i in 0..(size as usize).min(8) {
            if idx + i < MAX_INTERRUPTS {
                self.priority[idx + i] = ((value >> (i * 8)) & 0xFF) as u8;
            }
        }
    }

    fn set_bitmap_word(&mut self, offset: u64, base: u64, value: u32) {
        let idx = gicd_word_index(offset, base);
        self.set_enable_word_bits(idx, value);
    }

    fn clear_bitmap_word(&mut self, offset: u64, base: u64, value: u32) {
        let idx = gicd_word_index(offset, base);
        self.clear_enable_word_bits(idx, value);
    }
}
