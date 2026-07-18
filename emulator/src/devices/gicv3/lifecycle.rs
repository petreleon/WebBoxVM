use super::*;

impl Gicv3 {
    /// Atomically move one pending interrupt instance into the active state.
    pub fn acknowledge_interrupt_for_cpu(&mut self, cpu_id: usize, int_id: u32) -> bool {
        let Some(bit) = interrupt_bit(int_id) else {
            return false;
        };
        if int_id < SPI_FIRST {
            let Some(redistributor) = self.redistributors.get_mut(cpu_id) else {
                return false;
            };
            if redistributor.pending & bit == 0 || redistributor.active & bit != 0 {
                return false;
            }
            redistributor.pending &= !bit;
            redistributor.active |= bit;
            return true;
        }

        let idx = (int_id / 32) as usize;
        if idx >= INT_WORDS
            || !self.spi_targets_cpu(int_id, cpu_id)
            || self.pending[idx] & bit == 0
            || self.active[idx] & bit != 0
        {
            return false;
        }
        self.clear_pending_word_bits(idx, bit);
        self.active[idx] |= bit;
        true
    }

    /// Deactivate an interrupt without discarding a later pending instance.
    pub fn deactivate_interrupt_for_cpu(&mut self, cpu_id: usize, int_id: u32) -> bool {
        let Some(bit) = interrupt_bit(int_id) else {
            return false;
        };
        if int_id < SPI_FIRST {
            let Some(redistributor) = self.redistributors.get_mut(cpu_id) else {
                return false;
            };
            let was_active = redistributor.active & bit != 0;
            redistributor.active &= !bit;
            return was_active;
        }

        let idx = (int_id / 32) as usize;
        if cpu_id >= self.cpu_count() || idx >= INT_WORDS {
            return false;
        }
        let was_active = self.active[idx] & bit != 0;
        self.active[idx] &= !bit;
        was_active
    }

    pub fn is_active_for_cpu(&self, cpu_id: usize, int_id: u32) -> bool {
        let Some(bit) = interrupt_bit(int_id) else {
            return false;
        };
        if int_id < SPI_FIRST {
            return self
                .redistributors
                .get(cpu_id)
                .is_some_and(|redistributor| redistributor.active & bit != 0);
        }

        let idx = (int_id / 32) as usize;
        cpu_id < self.cpu_count() && idx < INT_WORDS && self.active[idx] & bit != 0
    }
}

fn interrupt_bit(int_id: u32) -> Option<u32> {
    (int_id < MAX_INTERRUPTS as u32).then(|| 1u32 << (int_id % 32))
}
