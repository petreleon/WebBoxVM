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

    pub fn irq_masked(&self) -> bool {
        self.bit(PSTATE_I_BIT)
    }

    pub fn with_irq_masked(mut self, masked: bool) -> Self {
        if masked {
            self.bits |= 1 << PSTATE_I_BIT;
        } else {
            self.bits &= !(1 << PSTATE_I_BIT);
        }
        self
    }

    pub fn daif(&self) -> u64 {
        self.bits & PSTATE_DAIF_MASK
    }

    pub fn with_daif(mut self, daif: u64) -> Self {
        self.bits = (self.bits & !PSTATE_DAIF_MASK) | (daif & PSTATE_DAIF_MASK);
        self
    }

    pub fn all_exceptions_masked(&self) -> bool {
        self.daif() == PSTATE_DAIF_MASK
    }

    pub fn with_all_exceptions_masked(self) -> Self {
        self.with_daif(PSTATE_DAIF_MASK)
    }

    // ── Condition flags ──

    pub fn n(&self) -> bool {
        self.bit(PSTATE_N_BIT)
    }
    pub fn z(&self) -> bool {
        self.bit(PSTATE_Z_BIT)
    }
    pub fn c(&self) -> bool {
        self.bit(PSTATE_C_BIT)
    }
    pub fn v(&self) -> bool {
        self.bit(PSTATE_V_BIT)
    }

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

    /// Whether the current EL's stack pointer (SP_ELx) is selected.
    pub fn sp_select(&self) -> bool {
        self.bit(PSTATE_SP_BIT)
    }

    pub fn with_sp_select(mut self, select_sp_elx: bool) -> Self {
        if select_sp_elx {
            self.bits |= PSTATE_SP_MASK;
        } else {
            self.bits &= !PSTATE_SP_MASK;
        }
        self
    }

    pub fn with_el1h(self) -> Self {
        self.with_el(1).with_sp_select(true)
    }

    /// Reset state required for a PSCI-started AArch64 secondary CPU.
    pub fn el1h_masked() -> Self {
        Self { bits: 0 }.with_el1h().with_all_exceptions_masked()
    }

    // ── Serialization ──

    /// Pack PSTATE into a u64 (SPSR_ELx format).
    pub fn to_u64(&self) -> u64 {
        self.bits
    }

    /// Unpack PSTATE from a u64 (SPSR_ELx format).
    pub fn from_u64(val: u64) -> Self {
        Self { bits: val }
    }

    // ── Private helpers ──

    fn bit(&self, shift: u32) -> bool {
        (self.bits >> shift) & 1 != 0
    }

    fn flag_bit(value: bool, shift: u32) -> u64 {
        (value as u64) << shift
    }
}

#[cfg(test)]
mod tests;
