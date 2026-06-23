//! ARM64 System Registers — the control knobs for the CPU and platform.
//!
//! These are the registers accessed by MRS (read) and MSR (write) instructions.
//! Each has a unique 16-bit ID composed as: op0:op1:CRn:CRm:op2.
//!
//! For a beginner: think of system registers as "configuration variables" that
//! control how the CPU behaves — whether the MMU is on, where the page tables
//! live, what the timer frequency is, etc.

use crate::constants::*;

mod defaults;
mod read;
mod timer;
mod write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemRegisters {
    // ── Memory management ──
    pub sctlr_el1: u64, // System Control Register — bit 0 enables MMU
    pub tcr_el1: u64,   // Translation Control Register — VA space size
    pub ttbr0_el1: u64, // Translation Table Base 0 — user-space page table root
    pub ttbr1_el1: u64, // Translation Table Base 1 — kernel-space page table root
    pub mair_el1: u64,  // Memory Attribute Indirection Register
    pub far_el1: u64,   // Fault Address Register

    // ── Exception handling ──
    pub vbar_el1: u64, // Vector Base Address — where exception handlers live
    pub spsr_el1: u64, // Saved Program Status Register — NZCV + EL before exception
    pub elr_el1: u64,  // Exception Link Register — return address after exception
    pub esr_el1: u64,  // Exception Syndrome Register — why the exception happened

    // ── Feature access ──
    pub cpacr_el1: u64, // Architectural Feature Access Control (FP/SIMD enable)
    pub fpcr: u64,      // Floating-point Control Register
    pub fpsr: u64,      // Floating-point Status Register

    // ── Generic Timer ──
    pub cntfrq_el0: u64,  // Counter frequency (62.5 MHz default)
    pub cycle_count: u64, // Emulated cycle counter — increments per instruction
    pub cntkctl_el1: u64, // Timer kernel control register

    // ── EL3 / secure world (used during boot stub) ──
    pub scr_el3: u64,  // Secure Configuration Register
    pub spsr_el3: u64, // Saved PSR for EL3
    pub elr_el3: u64,  // Exception Link for EL3

    // ── EL2 / hypervisor (used during boot stub) ──
    pub hcr_el2: u64,  // Hypervisor Configuration Register
    pub spsr_el2: u64, // Saved PSR for EL2
    pub elr_el2: u64,  // Exception Link for EL2

    // ── Thread / process ID registers (used by Linux for per-CPU variables) ──
    pub sp_el0: u64,
    pub sp_el1: u64,
    pub tpidr_el0: u64,
    pub tpidr_el1: u64,
    pub tpidrro_el0: u64,

    // ── GICv3 CPU interface (system register access) ──
    pub icc_pmr_el1: u64,  // Priority Mask
    pub icc_ctlr_el1: u64, // Control Register
    pub icc_sre_el1: u64,  // System Register Enable
    pub icc_iar1_el1: u64, // Interrupt Acknowledge

    // ── Timer control ──
    pub cntp_ctl_el0: u64, // Timer control: bit 0=enable, bit 1=mask, bit 2=status
    pub cntp_cval_el0: u64, // Timer compare value (absolute)
    pub cntp_tval_el0: u64, // Timer timer value (relative, decrements)
    pub cntv_ctl_el0: u64, // Virtual timer control
    pub cntv_cval_el0: u64, // Virtual timer compare value (absolute)
    pub cntv_tval_el0: u64, // Virtual timer value (relative)

    // ── Interrupt state (not real ARM registers, but emulator bookkeeping) ──
    pub irq_pending: bool,
    pub last_irq_id: u32,
}

impl SystemRegisters {
    pub fn cntp_enabled(&self) -> bool {
        self.cntp_ctl_el0 & TIMER_CTL_ENABLE != 0
    }

    pub fn cntp_unmasked(&self) -> bool {
        self.cntp_ctl_el0 & TIMER_CTL_IMASK == 0
    }

    pub fn cntp_expired(&self) -> bool {
        self.cntp_enabled() && self.cycle_count >= self.cntp_cval_el0
    }

    pub fn cntv_enabled(&self) -> bool {
        self.cntv_ctl_el0 & TIMER_CTL_ENABLE != 0
    }

    pub fn cntv_unmasked(&self) -> bool {
        self.cntv_ctl_el0 & TIMER_CTL_IMASK == 0
    }

    pub fn cntv_expired(&self) -> bool {
        self.cntv_enabled() && self.cycle_count >= self.cntv_cval_el0
    }

    pub fn timer_irq_check_needed(&self) -> bool {
        self.vbar_el1 != 0
            && (self.irq_pending
                || self.cntp_expired() && self.cntp_unmasked()
                || self.cntv_expired() && self.cntv_unmasked())
    }

    pub fn next_timer_deadline(&self) -> Option<u64> {
        let physical = (self.cntp_enabled() && self.cntp_unmasked())
            .then_some(self.cntp_cval_el0.max(self.cycle_count));
        let virtual_timer = (self.cntv_enabled() && self.cntv_unmasked())
            .then_some(self.cntv_cval_el0.max(self.cycle_count));
        match (physical, virtual_timer) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    pub(in crate::arch::arm64::system_regs) fn cntp_ctl_value(&self) -> u64 {
        let status = if self.cntp_expired() {
            TIMER_CTL_ISTATUS
        } else {
            0
        };
        (self.cntp_ctl_el0 & (TIMER_CTL_ENABLE | TIMER_CTL_IMASK)) | status
    }

    pub(in crate::arch::arm64::system_regs) fn cntv_ctl_value(&self) -> u64 {
        let status = if self.cntv_expired() {
            TIMER_CTL_ISTATUS
        } else {
            0
        };
        (self.cntv_ctl_el0 & (TIMER_CTL_ENABLE | TIMER_CTL_IMASK)) | status
    }
}
