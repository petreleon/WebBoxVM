//! System registers required for Linux boot.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemRegisters {
    pub sctlr_el1: u64,
    pub tcr_el1: u64,
    pub ttbr0_el1: u64,
    pub ttbr1_el1: u64,
    pub mair_el1: u64,
    pub vbar_el1: u64,
    pub spsr_el1: u64,
    pub elr_el1: u64,
    pub esr_el1: u64,
    pub far_el1: u64,
    pub cpacr_el1: u64,
    pub cntfrq_el0: u64,
    // Boot stub (EL3)
    pub scr_el3: u64,
    pub spsr_el3: u64,
    pub elr_el3: u64,
    // Boot stub (EL2)
    pub hcr_el2: u64,
    pub spsr_el2: u64,
    pub elr_el2: u64,

    // Thread/Stack registers
    pub sp_el0: u64,
    pub tpidr_el0: u64,
    pub tpidr_el1: u64,
    pub tpidrro_el0: u64,

    // Cycle counter (incremented per instruction)
    pub cycle_count: u64,

    // GICv3 CPU interface registers
    pub icc_pmr_el1: u64,   // Priority Mask
    pub icc_ctlr_el1: u64,  // Control Register
    pub icc_sre_el1: u64,   // System Register Enable
    pub icc_iar1_el1: u64,  // Interrupt Acknowledge

    // Generic Timer registers
    pub cntp_ctl_el0: u64,  // Timer control (ISTATUS, IMASK, ENABLE)
    pub cntp_cval_el0: u64, // Timer compare value
    pub cntp_tval_el0: u64, // Timer timer value (relative)

    // Interrupt state
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
            vbar_el1: 0,
            spsr_el1: 0,
            elr_el1: 0,
            esr_el1: 0,
            far_el1: 0,
            cpacr_el1: 0,
            cntfrq_el0: 62_500_000,
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
            cycle_count: 0,
            icc_pmr_el1: 0,
            icc_ctlr_el1: 0,
            icc_sre_el1: 0,
            icc_iar1_el1: 0x3FF, // spurious interrupt ID (1023)
            cntp_ctl_el0: 0,
            cntp_cval_el0: 0,
            cntp_tval_el0: 0,
            irq_pending: false,
            last_irq_id: 1023,
        }
    }
}

impl SystemRegisters {
    /// Read a system register by its 15-bit `{o0, op1, CRn, CRm, op2}` ID.
    pub fn read_sys_reg(&mut self, sysreg_id: u16, current_el: u8) -> u64 {
        match sysreg_id {
            0x4208 => self.sp_el0,
            0x5E82 => self.tpidr_el0,
            0x4684 => self.tpidr_el1,
            0x5E83 => self.tpidrro_el0,
            0x4080 => self.sctlr_el1,
            0x4102 => self.tcr_el1,
            0x4100 => self.ttbr0_el1,
            0x4101 => self.ttbr1_el1,
            0x4510 => self.mair_el1,
            0x4600 => self.vbar_el1,
            0x4200 => self.spsr_el1,
            0x4201 => self.elr_el1,
            0x4290 => self.esr_el1,
            0x4300 => self.far_el1,
            0x4082 => self.cpacr_el1,
            0x5F00 => self.cntfrq_el0,
            0x7088 => self.scr_el3,
            0x7200 => self.spsr_el3,
            0x7201 => self.elr_el3,
            0x6088 => self.hcr_el2,
            0x6200 => self.spsr_el2,
            0x6201 => self.elr_el2,

            // Read-only / feature / status registers — ARMv8.0-A values
            0x4000 => 0x410FD083, // MIDR_EL1 (Cortex-A72 r0p3)
            0x4005 => 0x80000000, // MPIDR_EL1 (Single core, cluster 0, core 0)
            0x4212 => (current_el as u64) << 2, // CurrentEL
            // ID registers: indicate ARMv8.0-A, AArch64 at all ELs, FP+SIMD, no crypto (use software)
            0x4020 => 0x0000000000000011, // ID_AA64PFR0_EL1: EL0/1/2/3=A64-only, FP+SIMD
            0x4021 => 0x0000000000000000, // ID_AA64PFR1_EL1
            0x4028 => 0x0000000000000000, // ID_AA64PFR2_EL1
            0x4030 => 0x0000000000103106, // ID_AA64DFR0_EL1: debug v8, PMU v3
            0x4031 => 0x0000000000000000, // ID_AA64DFR1_EL1
            0x4032 => 0x0000000000101001, // ID_AA64ISAR0_EL1: AES+PMULL+SHA1+SHA256+CRC32
            0x4033 => 0x0000000000000121, // ID_AA64ISAR1_EL1: DP+LRCPC+FCMA+JSCVT
            0x4034 => 0x0000000000000000, // ID_AA64ISAR2_EL1
            0x4038 => 0x0000000000001122, // ID_AA64MMFR0_EL1: 4K+64K granule, 48-bit PA
            0x4039 => 0x0000000000000000, // ID_AA64MMFR1_EL1
            0x403A => 0x0000000000000000, // ID_AA64MMFR2_EL1
            0x5801 => 0x8444c004, // CTR_EL0 (Cache Type Register)
            0x5807 => 0x0000000000000010, // DCZID_EL0 (DC ZVA block size = 16 bytes)
            // Generic Timer: monotonic counter for spinlock backoff / udelay
            0x5F01 => self.cycle_count, // CNTPCT_EL0 (physical counter)
            0x5F02 => self.cycle_count, // CNTVCT_EL0 (virtual counter)
            // GICv3 CPU interface
            0x4230 => self.icc_pmr_el1,  // ICC_PMR_EL1
            0x4234 => self.icc_ctlr_el1, // ICC_CTLR_EL1
            0x4665 => self.icc_sre_el1,  // ICC_SRE_EL1
            0x4660 => { // ICC_IAR1_EL1 — acknowledge interrupt
                if self.irq_pending {
                    let id = self.last_irq_id as u64;
                    self.irq_pending = false;
                    id
                } else {
                    0x3FF // spurious
                }
            }
            // Generic Timer control
            0x5F10 => self.cntp_tval_el0,
            0x5F11 => self.cntp_ctl_el0,
            0x5F12 => self.cntp_cval_el0,

            _ => 0,
        }
    }

    /// Write a system register by its 15-bit `{o0, op1, CRn, CRm, op2}` ID.
    pub fn write_sys_reg(&mut self, sysreg_id: u16, val: u64) {
        match sysreg_id {
            0x4208 => self.sp_el0 = val,
            0x5E82 => self.tpidr_el0 = val,
            0x4684 => self.tpidr_el1 = val,
            0x5E83 => self.tpidrro_el0 = val,
            0x4080 => self.sctlr_el1 = val,
            0x4102 => self.tcr_el1 = val,
            0x4100 => self.ttbr0_el1 = val,
            0x4101 => self.ttbr1_el1 = val,
            0x4510 => self.mair_el1 = val,
            0x4600 => self.vbar_el1 = val,
            0x4200 => self.spsr_el1 = val,
            0x4201 => self.elr_el1 = val,
            0x4290 => self.esr_el1 = val,
            0x4300 => self.far_el1 = val,
            0x4082 => self.cpacr_el1 = val,
            0x5F00 => self.cntfrq_el0 = val,
            // GICv3 CPU interface
            0x4230 => { self.icc_pmr_el1 = val; }
            0x4234 => { self.icc_ctlr_el1 = val; }
            0x4665 => { self.icc_sre_el1 = val; }
            0x4661 => { // ICC_EOIR1_EL1 — end of interrupt
                self.irq_pending = false;
                self.last_irq_id = 1023;
            }
            // Generic Timer
            0x5F10 => { // CNTP_TVAL_EL0
                self.cntp_tval_el0 = val;
                self.cntp_cval_el0 = self.cycle_count.wrapping_add(val as u32 as u64);
            }
            0x5F11 => { self.cntp_ctl_el0 = val; }
            0x5F12 => { self.cntp_cval_el0 = val; }
            0x7088 => self.scr_el3 = val,
            0x7200 => self.spsr_el3 = val,
            0x7201 => self.elr_el3 = val,
            0x6088 => self.hcr_el2 = val,
            0x6200 => self.spsr_el2 = val,
            0x6201 => self.elr_el2 = val,

            // Read-only / feature / status registers are no-op
            _ => {}
        }
    }
}
