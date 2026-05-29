//! Processor State — NZCV condition flags, Exception Level, and interrupt masks.
//!
//! In a real ARM64 processor this is the PSTATE register.  It holds:
//!   - NZCV flags (Negative, Zero, Carry, oVerflow) — set by arithmetic ops
//!   - Exception Level (EL0–EL3) — the privilege ring the CPU is running in
//!   - Interrupt masks (I, F, A, D) — whether IRQs/FIQs are blocked
//!
//! We store it as a flat u64 matching the SPSR_ELx format for easy save/restore
//! during exception entry/return.

use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProcessorState {
    bits: u64,
}

impl ProcessorState {
    pub fn new() -> Self {
        // ARM cores boot at the highest privilege level (EL3) with interrupts masked.
        Self { bits: 0 }.with_el(MAX_EL).with_irq_masked(true)
    }

    // ── Interrupt mask ──

    pub fn irq_masked(&self) -> bool { self.bit(PSTATE_I_BIT) }

    pub fn with_irq_masked(mut self, masked: bool) -> Self {
        if masked { self.bits |= 1 << PSTATE_I_BIT; }
        else      { self.bits &= !(1 << PSTATE_I_BIT); }
        self
    }

    // ── Condition flags ──

    pub fn n(&self) -> bool { self.bit(PSTATE_N_BIT) }
    pub fn z(&self) -> bool { self.bit(PSTATE_Z_BIT) }
    pub fn c(&self) -> bool { self.bit(PSTATE_C_BIT) }
    pub fn v(&self) -> bool { self.bit(PSTATE_V_BIT) }

    /// Set all four NZCV flags at once. Clears the existing flags first.
    pub fn set_nzcv(&mut self, n: bool, z: bool, c: bool, v: bool) {
        self.bits = (self.bits & !PSTATE_NZCV_MASK)
            | Self::flag_bit(n, PSTATE_N_BIT)
            | Self::flag_bit(z, PSTATE_Z_BIT)
            | Self::flag_bit(c, PSTATE_C_BIT)
            | Self::flag_bit(v, PSTATE_V_BIT);
    }

    // ── Exception level ──

    /// Current Exception Level: 0 (user), 1 (kernel), 2 (hypervisor), 3 (secure monitor).
    pub fn el(&self) -> u8 {
        ((self.bits >> PSTATE_EL_SHIFT) & 3) as u8
    }

    /// Return a copy with the Exception Level changed.
    pub fn with_el(mut self, level: u8) -> Self {
        assert!(level <= MAX_EL, "EL must be 0–{}, got {}", MAX_EL, level);
        self.bits = (self.bits & !PSTATE_EL_MASK) | ((level as u64) << PSTATE_EL_SHIFT);
        self
    }

    // ── Serialization ──

    /// Pack PSTATE into a u64 (SPSR_ELx format).
    pub fn to_u64(&self) -> u64 { self.bits }

    /// Unpack PSTATE from a u64 (SPSR_ELx format).
    pub fn from_u64(val: u64) -> Self { Self { bits: val } }

    // ── Private helpers ──

    fn bit(&self, shift: u32) -> bool {
        (self.bits >> shift) & 1 != 0
    }

    fn flag_bit(value: bool, shift: u32) -> u64 {
        (value as u64) << shift
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_el3() {
        let p = ProcessorState::new();
        assert_eq!(p.el(), MAX_EL);
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
