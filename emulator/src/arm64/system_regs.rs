//! ARM64 System Registers — the control knobs for the CPU and platform.
//!
//! These are the registers accessed by MRS (read) and MSR (write) instructions.
//! Each has a unique 16-bit ID composed as: op0:op1:CRn:CRm:op2.
//!
//! For a beginner: think of system registers as "configuration variables" that
//! control how the CPU behaves — whether the MMU is on, where the page tables
//! live, what the timer frequency is, etc.

use crate::constants::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemRegisters {
    // ── Memory management ──
    pub sctlr_el1: u64,   // System Control Register — bit 0 enables MMU
    pub tcr_el1: u64,     // Translation Control Register — VA space size
    pub ttbr0_el1: u64,   // Translation Table Base 0 — user-space page table root
    pub ttbr1_el1: u64,   // Translation Table Base 1 — kernel-space page table root
    pub mair_el1: u64,    // Memory Attribute Indirection Register
    pub far_el1: u64,     // Fault Address Register

    // ── Exception handling ──
    pub vbar_el1: u64,    // Vector Base Address — where exception handlers live
    pub spsr_el1: u64,    // Saved Program Status Register — NZCV + EL before exception
    pub elr_el1: u64,     // Exception Link Register — return address after exception
    pub esr_el1: u64,     // Exception Syndrome Register — why the exception happened

    // ── Feature access ──
    pub cpacr_el1: u64,   // Architectural Feature Access Control (FP/SIMD enable)

    // ── Generic Timer ──
    pub cntfrq_el0: u64,  // Counter frequency (62.5 MHz default)
    pub cycle_count: u64, // Emulated cycle counter — increments per instruction

    // ── EL3 / secure world (used during boot stub) ──
    pub scr_el3: u64,     // Secure Configuration Register
    pub spsr_el3: u64,    // Saved PSR for EL3
    pub elr_el3: u64,     // Exception Link for EL3

    // ── EL2 / hypervisor (used during boot stub) ──
    pub hcr_el2: u64,     // Hypervisor Configuration Register
    pub spsr_el2: u64,    // Saved PSR for EL2
    pub elr_el2: u64,     // Exception Link for EL2

    // ── Thread / process ID registers (used by Linux for per-CPU variables) ──
    pub sp_el0: u64,
    pub tpidr_el0: u64,
    pub tpidr_el1: u64,
    pub tpidrro_el0: u64,

    // ── GICv3 CPU interface (system register access) ──
    pub icc_pmr_el1: u64,   // Priority Mask
    pub icc_ctlr_el1: u64,  // Control Register
    pub icc_sre_el1: u64,   // System Register Enable
    pub icc_iar1_el1: u64,  // Interrupt Acknowledge

    // ── Timer control ──
    pub cntp_ctl_el0: u64,   // Timer control: bit 0=enable, bit 1=mask, bit 2=status
    pub cntp_cval_el0: u64,  // Timer compare value (absolute)
    pub cntp_tval_el0: u64,  // Timer timer value (relative, decrements)

    // ── Interrupt state (not real ARM registers, but emulator bookkeeping) ──
    pub irq_pending: bool,
    pub last_irq_id: u32,
}

impl Default for SystemRegisters {
    fn default() -> Self {
        Self {
            sctlr_el1: 0,
            tcr_el1: 0,
            ttbr0_el1: 0,
            ttbr1_el1: 0,
            mair_el1: 0,
            far_el1: 0,
            vbar_el1: 0,
            spsr_el1: 0,
            elr_el1: 0,
            esr_el1: 0,
            cpacr_el1: 0,
            cntfrq_el0: TIMER_FREQ_HZ,
            cycle_count: 0,
            scr_el3: 0,
            spsr_el3: 0,
            elr_el3: 0,
            hcr_el2: 0,
            spsr_el2: 0,
            elr_el2: 0,
            sp_el0: 0,
            tpidr_el0: 0,
            tpidr_el1: 0,
            tpidrro_el0: 0,
            icc_pmr_el1: 0,
            icc_ctlr_el1: 0,
            icc_sre_el1: 0,
            icc_iar1_el1: GIC_SPURIOUS_INTERRUPT,
            cntp_ctl_el0: 0,
            cntp_cval_el0: 0,
            cntp_tval_el0: 0,
            irq_pending: false,
            last_irq_id: GIC_SPURIOUS_INTERRUPT as u32,
        }
    }
}

impl SystemRegisters {
    /// Read a system register by its 15-bit ID.
    pub fn read_sys_reg(&mut self, sysreg_id: u16, current_el: u8) -> u64 {
        match sysreg_id {
            SYSREG_SP_EL0      => self.sp_el0,
            SYSREG_TPIDR_EL0   => self.tpidr_el0,
            SYSREG_TPIDR_EL1   => self.tpidr_el1,
            SYSREG_TPIDRRO_EL0 => self.tpidrro_el0,

            // ── MMU system registers ──
            SYSREG_SCTLR_EL1   => self.sctlr_el1,
            SYSREG_TCR_EL1     => self.tcr_el1,
            SYSREG_TTBR0_EL1   => self.ttbr0_el1,
            SYSREG_TTBR1_EL1   => self.ttbr1_el1,
            SYSREG_MAIR_EL1    => self.mair_el1,

            // ── Exception system registers ──
            SYSREG_VBAR_EL1    => self.vbar_el1,
            SYSREG_SPSR_EL1    => self.spsr_el1,
            SYSREG_ELR_EL1     => self.elr_el1,
            SYSREG_ESR_EL1     => self.esr_el1,
            SYSREG_FAR_EL1     => self.far_el1,
            SYSREG_CPACR_EL1   => self.cpacr_el1,

            // ── Timer ──
            SYSREG_CNTFRQ_EL0      => self.cntfrq_el0,
            SYSREG_CNTPCT_EL0      => self.cycle_count,
            SYSREG_CNTVCT_EL0      => self.cycle_count,
            SYSREG_CNTP_TVAL_EL0   => self.cntp_tval_el0,
            SYSREG_CNTP_CTL_EL0    => self.cntp_ctl_el0,
            SYSREG_CNTP_CVAL_EL0   => self.cntp_cval_el0,

            // ── EL3 / secure world ──
            SYSREG_SCR_EL3    => self.scr_el3,
            SYSREG_SPSR_EL3   => self.spsr_el3,
            SYSREG_ELR_EL3    => self.elr_el3,

            // ── EL2 / hypervisor ──
            SYSREG_HCR_EL2    => self.hcr_el2,
            SYSREG_SPSR_EL2   => self.spsr_el2,
            SYSREG_ELR_EL2    => self.elr_el2,

            // ── Read-only feature / identification registers ──
            SYSREG_MIDR_EL1            => MIDR_CORTEX_A72_R0P3,
            SYSREG_MPIDR_EL1           => MPIDR_SINGLE_CORE,
            SYSREG_CURRENTEL           => (current_el as u64) << PSTATE_EL_SHIFT,
            SYSREG_ID_AA64PFR0_EL1     => ID_AA64PFR0_EL1_VAL,
            SYSREG_ID_AA64PFR1_EL1 | SYSREG_ID_AA64PFR2_EL1
                | SYSREG_ID_AA64DFR1_EL1 | SYSREG_ID_AA64ISAR2_EL1
                | SYSREG_ID_AA64MMFR1_EL1 | SYSREG_ID_AA64MMFR2_EL1 => 0,
            SYSREG_ID_AA64DFR0_EL1     => ID_AA64DFR0_EL1_VAL,
            SYSREG_ID_AA64ISAR0_EL1    => ID_AA64ISAR0_EL1_VAL,
            SYSREG_ID_AA64ISAR1_EL1    => ID_AA64ISAR1_EL1_VAL,
            SYSREG_ID_AA64MMFR0_EL1    => ID_AA64MMFR0_EL1_VAL,
            SYSREG_CTR_EL0             => CTR_EL0_VAL,
            SYSREG_DCZID_EL0           => DCZID_EL0_VAL,

            // ── GICv3 CPU interface ──
            SYSREG_ICC_PMR_EL1  => self.icc_pmr_el1,
            SYSREG_ICC_CTLR_EL1 => self.icc_ctlr_el1,
            SYSREG_ICC_SRE_EL1  => self.icc_sre_el1,

            SYSREG_ICC_IAR1_EL1 => {
                // Acknowledge interrupt — consume the pending IRQ
                if self.irq_pending {
                    let id = self.last_irq_id as u64;
                    self.irq_pending = false;
                    id
                } else {
                    GIC_SPURIOUS_INTERRUPT
                }
            }

            _ => 0, // unknown register → reads as 0
        }
    }

    /// Write a system register by its 15-bit ID.
    pub fn write_sys_reg(&mut self, sysreg_id: u16, val: u64) {
        match sysreg_id {
            SYSREG_SP_EL0      => self.sp_el0 = val,
            SYSREG_TPIDR_EL0   => self.tpidr_el0 = val,
            SYSREG_TPIDR_EL1   => self.tpidr_el1 = val,
            SYSREG_TPIDRRO_EL0 => self.tpidrro_el0 = val,

            SYSREG_SCTLR_EL1   => self.sctlr_el1 = val,
            SYSREG_TCR_EL1     => self.tcr_el1 = val,
            SYSREG_TTBR0_EL1   => self.ttbr0_el1 = val,
            SYSREG_TTBR1_EL1   => self.ttbr1_el1 = val,
            SYSREG_MAIR_EL1    => self.mair_el1 = val,
            SYSREG_VBAR_EL1    => self.vbar_el1 = val,
            SYSREG_SPSR_EL1    => self.spsr_el1 = val,
            SYSREG_ELR_EL1     => self.elr_el1 = val,
            SYSREG_ESR_EL1     => self.esr_el1 = val,
            SYSREG_FAR_EL1     => self.far_el1 = val,
            SYSREG_CPACR_EL1   => self.cpacr_el1 = val,
            SYSREG_CNTFRQ_EL0  => self.cntfrq_el0 = val,

            // GICv3 CPU interface
            SYSREG_ICC_PMR_EL1  => self.icc_pmr_el1 = val,
            SYSREG_ICC_CTLR_EL1 => self.icc_ctlr_el1 = val,
            SYSREG_ICC_SRE_EL1  => self.icc_sre_el1 = val,
            SYSREG_ICC_EOIR1_EL1 => {
                self.irq_pending = false;
                self.last_irq_id = GIC_SPURIOUS_INTERRUPT as u32;
            }

            // Generic Timer
            SYSREG_CNTP_TVAL_EL0 => {
                self.cntp_tval_el0 = val;
                self.cntp_cval_el0 = self.cycle_count.wrapping_add(val as u32 as u64);
            }
            SYSREG_CNTP_CTL_EL0 => self.cntp_ctl_el0 = val,
            SYSREG_CNTP_CVAL_EL0 => self.cntp_cval_el0 = val,

            // DAIF: bits [9:6] = D, A, I, F.  Bit 7 = IRQ mask.
            SYSREG_DAIF => {} // handled in execute.rs MSR path
            SYSREG_SCR_EL3  => self.scr_el3 = val,
            SYSREG_SPSR_EL3 => self.spsr_el3 = val,
            SYSREG_ELR_EL3  => self.elr_el3 = val,
            SYSREG_HCR_EL2  => self.hcr_el2 = val,
            SYSREG_SPSR_EL2 => self.spsr_el2 = val,
            SYSREG_ELR_EL2  => self.elr_el2 = val,

            _ => {} // unknown register — silently ignored
        }
    }
}
