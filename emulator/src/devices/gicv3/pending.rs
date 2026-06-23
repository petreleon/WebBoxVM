use super::*;

impl Gicv3 {
    pub fn set_pending(&mut self, int_id: u32) {
        let idx = (int_id / 32) as usize;
        let bit = 1u32 << (int_id % 32);
        self.set_pending_word_bits(idx, bit);
    }

    pub fn clear_pending(&mut self, int_id: u32) {
        let idx = (int_id / 32) as usize;
        let bit = 1u32 << (int_id % 32);
        self.clear_pending_word_bits(idx, bit);
    }

    pub fn enable_interrupt(&mut self, int_id: u32) {
        let idx = (int_id / 32) as usize;
        let bit = 1u32 << (int_id % 32);
        self.set_enable_word_bits(idx, bit);
    }

    pub fn is_pending(&self, int_id: u32) -> bool {
        let idx = (int_id / 32) as usize;
        let bit = 1u32 << (int_id % 32);
        idx < INT_WORDS && (self.pending[idx] & bit) != 0
    }

    pub fn enable_word(&self, idx: usize) -> u32 {
        self.enable.get(idx).copied().unwrap_or(0)
    }

    pub fn pending_word(&self, idx: usize) -> u32 {
        self.pending.get(idx).copied().unwrap_or(0)
    }

    pub fn has_pending_enabled(&self) -> bool {
        self.pending_enabled_words != 0
    }

    pub fn next_pending_enabled(&self) -> Option<u32> {
        if !self.has_pending_enabled() {
            return None;
        }

        let active_words = self.pending_enabled_words;
        let idx = active_words.trailing_zeros() as usize;
        let active = self.pending_enabled[idx];
        debug_assert_ne!(active, 0);
        Some((idx as u32) * 32 + active.trailing_zeros())
    }

    pub(super) fn set_pending_word_bits(&mut self, idx: usize, bits: u32) {
        if idx < INT_WORDS {
            self.pending[idx] |= bits;
            self.refresh_pending_enabled_word(idx);
        }
    }

    pub(super) fn clear_pending_word_bits(&mut self, idx: usize, bits: u32) {
        if idx < INT_WORDS {
            self.pending[idx] &= !bits;
            self.refresh_pending_enabled_word(idx);
        }
    }

    pub(super) fn set_enable_word_bits(&mut self, idx: usize, bits: u32) {
        if idx < INT_WORDS {
            self.enable[idx] |= bits;
            self.refresh_pending_enabled_word(idx);
        }
    }

    pub(super) fn clear_enable_word_bits(&mut self, idx: usize, bits: u32) {
        if idx < INT_WORDS {
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
