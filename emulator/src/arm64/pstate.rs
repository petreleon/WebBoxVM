//! Processor state: NZCV flags, exception level, interrupt masks.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProcessorState {
    bits: u64,
}

impl ProcessorState {
    const N_SHIFT: u32 = 31;
    const Z_SHIFT: u32 = 30;
    const C_SHIFT: u32 = 29;
    const V_SHIFT: u32 = 28;
    const EL_SHIFT: u32 = 2;

    pub fn new() -> Self {
        Self { bits: 0 }.with_el(3) // Boot at EL3
    }

    // --- Condition flags ---

    pub fn n(&self) -> bool { self.bit(Self::N_SHIFT) }
    pub fn z(&self) -> bool { self.bit(Self::Z_SHIFT) }
    pub fn c(&self) -> bool { self.bit(Self::C_SHIFT) }
    pub fn v(&self) -> bool { self.bit(Self::V_SHIFT) }

    pub fn set_nzcv(&mut self, n: bool, z: bool, c: bool, v: bool) {
        self.bits = (self.bits & !0xF000_0000)
            | Self::flag(n, Self::N_SHIFT)
            | Self::flag(z, Self::Z_SHIFT)
            | Self::flag(c, Self::C_SHIFT)
            | Self::flag(v, Self::V_SHIFT);
    }

    // --- Exception level ---

    pub fn el(&self) -> u8 {
        ((self.bits >> Self::EL_SHIFT) & 3) as u8
    }

    pub fn with_el(mut self, level: u8) -> Self {
        assert!(level <= 3, "EL must be 0-3, got {}", level);
        self.bits = (self.bits & !(3 << Self::EL_SHIFT)) | ((level as u64) << Self::EL_SHIFT);
        self
    }

    // --- Private helpers ---

    fn bit(&self, shift: u32) -> bool {
        (self.bits >> shift) & 1 != 0
    }

    fn flag(v: bool, shift: u32) -> u64 {
        (v as u64) << shift
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_el3() {
        let p = ProcessorState::new();
        assert_eq!(p.el(), 3);
    }

    #[test]
    fn nzcv_roundtrip() {
        let mut p = ProcessorState::new();
        p.set_nzcv(true, false, true, false);
        assert!(p.n() && !p.z() && p.c() && !p.v());
    }

    #[test]
    fn el_transition() {
        let p = ProcessorState::new().with_el(1);
        assert_eq!(p.el(), 1);
    }
}
