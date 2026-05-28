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
        }
    }
}

impl SystemRegisters {
    /// Read a system register by its 15-bit `{o0, op1, CRn, CRm, op2}` ID.
    pub fn read_sys_reg(&self, sysreg_id: u16, current_el: u8) -> u64 {
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
            0x5F01 => 0, // CNTPCT_EL0 (physical counter, returns 0 — no timer emulation yet)
            0x5F02 => 0, // CNTVCT_EL0 (virtual counter)

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
