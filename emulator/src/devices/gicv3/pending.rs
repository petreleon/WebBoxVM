use super::*;

impl Gicv3 {
    /// Set an interrupt pending for CPU0 (private interrupt) or globally (SPI).
    pub fn set_pending(&mut self, int_id: u32) {
        self.set_pending_for_cpu(0, int_id);
    }

    pub fn set_pending_for_cpu(&mut self, cpu_id: usize, int_id: u32) {
        let bit = 1u32.checked_shl(int_id % 32).unwrap_or(0);
        if int_id < SPI_FIRST {
            if let Some(redistributor) = self.redistributors.get_mut(cpu_id) {
                redistributor.pending |= bit;
            }
            return;
        }
        self.set_pending_word_bits((int_id / 32) as usize, bit);
    }

    /// Clear an interrupt for CPU0 (private interrupt) or globally (SPI).
    pub fn clear_pending(&mut self, int_id: u32) {
        self.clear_pending_for_cpu(0, int_id);
    }

    pub fn clear_pending_for_cpu(&mut self, cpu_id: usize, int_id: u32) {
        let bit = 1u32.checked_shl(int_id % 32).unwrap_or(0);
        if int_id < SPI_FIRST {
            if let Some(redistributor) = self.redistributors.get_mut(cpu_id) {
                redistributor.pending &= !bit;
            }
            return;
        }
        self.clear_pending_word_bits((int_id / 32) as usize, bit);
    }

    /// Enable an interrupt for CPU0 (private interrupt) or globally (SPI).
    pub fn enable_interrupt(&mut self, int_id: u32) {
        self.enable_interrupt_for_cpu(0, int_id);
    }

    pub fn enable_interrupt_for_cpu(&mut self, cpu_id: usize, int_id: u32) {
        let bit = 1u32.checked_shl(int_id % 32).unwrap_or(0);
        if int_id < SPI_FIRST {
            if let Some(redistributor) = self.redistributors.get_mut(cpu_id) {
                redistributor.enable |= bit;
            }
            return;
        }
        self.set_enable_word_bits((int_id / 32) as usize, bit);
    }

    pub fn is_pending(&self, int_id: u32) -> bool {
        self.is_pending_for_cpu(0, int_id)
    }

    pub fn is_pending_for_cpu(&self, cpu_id: usize, int_id: u32) -> bool {
        let bit = 1u32.checked_shl(int_id % 32).unwrap_or(0);
        if int_id < SPI_FIRST {
            return self
                .redistributors
                .get(cpu_id)
                .is_some_and(|redistributor| redistributor.pending & bit != 0);
        }
        let idx = (int_id / 32) as usize;
        idx < INT_WORDS && self.pending[idx] & bit != 0
    }

    pub fn enable_word(&self, idx: usize) -> u32 {
        if idx == 0 {
            return self
                .redistributors
                .first()
                .map_or(0, |redistributor| redistributor.enable);
        }
        self.enable.get(idx).copied().unwrap_or(0)
    }

    pub fn pending_word(&self, idx: usize) -> u32 {
        if idx == 0 {
            return self
                .redistributors
                .first()
                .map_or(0, |redistributor| redistributor.pending);
        }
        self.pending.get(idx).copied().unwrap_or(0)
    }

    pub fn has_pending_enabled(&self) -> bool {
        self.has_pending_enabled_for_cpu(0)
    }

    pub fn has_pending_enabled_for_cpu(&self, cpu_id: usize) -> bool {
        self.next_pending_enabled_for_cpu(cpu_id).is_some()
    }

    pub fn next_pending_enabled(&self) -> Option<u32> {
        self.next_pending_enabled_for_cpu(0)
    }

    /// Return the lowest enabled pending interrupt deliverable to `cpu_id`.
    pub fn next_pending_enabled_for_cpu(&self, cpu_id: usize) -> Option<u32> {
        let redistributor = self.redistributors.get(cpu_id)?;
        let private = redistributor.pending_enabled();
        if private != 0 {
            return Some(private.trailing_zeros());
        }

        let mut active_words = self.pending_enabled_words & !1;
        while active_words != 0 {
            let idx = active_words.trailing_zeros() as usize;
            let mut candidates = self.pending_enabled[idx] & !self.active[idx];
            while candidates != 0 {
                let bit = candidates.trailing_zeros();
                let int_id = (idx as u32) * 32 + bit;
                if self.spi_targets_cpu(int_id, cpu_id) {
                    return Some(int_id);
                }
                candidates &= !(1u32 << bit);
            }
            active_words &= !(1u32 << idx);
        }
        None
    }

    pub(super) fn set_pending_word_bits(&mut self, idx: usize, bits: u32) {
        if idx == 0 {
            self.redistributors[0].pending |= bits;
        } else if idx < INT_WORDS {
            self.pending[idx] |= bits;
            self.refresh_pending_enabled_word(idx);
        }
    }

    pub(super) fn clear_pending_word_bits(&mut self, idx: usize, bits: u32) {
        if idx == 0 {
            self.redistributors[0].pending &= !bits;
        } else if idx < INT_WORDS {
            self.pending[idx] &= !bits;
            self.refresh_pending_enabled_word(idx);
        }
    }

    pub(super) fn set_enable_word_bits(&mut self, idx: usize, bits: u32) {
        if idx == 0 {
            self.redistributors[0].enable |= bits;
        } else if idx < INT_WORDS {
            self.enable[idx] |= bits;
            self.refresh_pending_enabled_word(idx);
        }
    }

    pub(super) fn clear_enable_word_bits(&mut self, idx: usize, bits: u32) {
        if idx == 0 {
            self.redistributors[0].enable &= !bits;
        } else if idx < INT_WORDS {
            self.enable[idx] &= !bits;
            self.refresh_pending_enabled_word(idx);
        }
    }

    fn refresh_pending_enabled_word(&mut self, idx: usize) {
        let active = self.pending[idx] & self.enable[idx];
        self.pending_enabled[idx] = active;
        let word_bit = 1u32 << idx;
        if active == 0 {
            self.pending_enabled_words &= !word_bit;
        } else {
            self.pending_enabled_words |= word_bit;
        }
    }
}
