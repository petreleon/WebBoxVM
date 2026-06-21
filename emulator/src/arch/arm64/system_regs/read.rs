use super::timer::timer_tval;
use super::*;

impl SystemRegisters {
    /// Read a system register by its 15-bit ID.
    pub fn read_sys_reg(&mut self, sysreg_id: u16, current_el: u8) -> u64 {
        match sysreg_id {
            SYSREG_SP_EL0 => self.sp_el0,
            SYSREG_SP_EL1 => self.sp_el1,
            SYSREG_TPIDR_EL0 => self.tpidr_el0,
            SYSREG_TPIDR_EL1 => self.tpidr_el1,
            SYSREG_TPIDRRO_EL0 => self.tpidrro_el0,
            SYSREG_SCTLR_EL1 => self.sctlr_el1,
            SYSREG_TCR_EL1 => self.tcr_el1,
            SYSREG_TTBR0_EL1 => self.ttbr0_el1,
            SYSREG_TTBR1_EL1 => self.ttbr1_el1,
            SYSREG_MAIR_EL1 => self.mair_el1,
            SYSREG_VBAR_EL1 => self.vbar_el1,
            SYSREG_SPSR_EL1 => self.spsr_el1,
            SYSREG_ELR_EL1 => self.elr_el1,
            SYSREG_ESR_EL1 => self.esr_el1,
            SYSREG_FAR_EL1 => self.far_el1,
            SYSREG_CPACR_EL1 => self.cpacr_el1,
            SYSREG_FPCR => self.fpcr,
            SYSREG_FPSR => self.fpsr,
            SYSREG_CNTFRQ_EL0 => self.cntfrq_el0,
            SYSREG_CNTPCT_EL0 => self.cycle_count,
            SYSREG_CNTVCT_EL0 => self.cycle_count,
            SYSREG_CNTKCTL_EL1 => self.cntkctl_el1,
            SYSREG_CNTP_TVAL_EL0 => timer_tval(self.cntp_cval_el0, self.cycle_count),
            SYSREG_CNTP_CTL_EL0 => self.cntp_ctl_value(),
            SYSREG_CNTP_CVAL_EL0 => self.cntp_cval_el0,
            SYSREG_CNTV_TVAL_EL0 => timer_tval(self.cntv_cval_el0, self.cycle_count),
            SYSREG_CNTV_CTL_EL0 => self.cntv_ctl_value(),
            SYSREG_CNTV_CVAL_EL0 => self.cntv_cval_el0,
            SYSREG_SCR_EL3 => self.scr_el3,
            SYSREG_SPSR_EL3 => self.spsr_el3,
            SYSREG_ELR_EL3 => self.elr_el3,
            SYSREG_HCR_EL2 => self.hcr_el2,
            SYSREG_SPSR_EL2 => self.spsr_el2,
            SYSREG_ELR_EL2 => self.elr_el2,
            SYSREG_MIDR_EL1 => MIDR_CORTEX_A72_R0P3,
            SYSREG_MPIDR_EL1 => MPIDR_SINGLE_CORE,
            SYSREG_CURRENTEL => (current_el as u64) << PSTATE_EL_SHIFT,
            SYSREG_ID_AA64PFR0_EL1 => ID_AA64PFR0_EL1_VAL,
            SYSREG_ID_AA64PFR1_EL1
            | SYSREG_ID_AA64PFR2_EL1
            | SYSREG_ID_AA64DFR1_EL1
            | SYSREG_ID_AA64ISAR2_EL1
            | SYSREG_ID_AA64MMFR2_EL1 => 0,
            SYSREG_ID_AA64DFR0_EL1 => ID_AA64DFR0_EL1_VAL,
            SYSREG_ID_AA64ISAR0_EL1 => ID_AA64ISAR0_EL1_VAL,
            SYSREG_ID_AA64ISAR1_EL1 => ID_AA64ISAR1_EL1_VAL,
            SYSREG_ID_AA64MMFR0_EL1 => ID_AA64MMFR0_EL1_VAL,
            SYSREG_ID_AA64MMFR1_EL1 => ID_AA64MMFR1_EL1_VAL,
            SYSREG_CTR_EL0 => CTR_EL0_VAL,
            SYSREG_DCZID_EL0 => DCZID_EL0_VAL,
            SYSREG_ICC_PMR_EL1 => self.icc_pmr_el1,
            SYSREG_ICC_CTLR_EL1 => self.icc_ctlr_el1,
            SYSREG_ICC_SRE_EL1 => self.icc_sre_el1,
            SYSREG_ICC_IAR1_EL1 => self.read_interrupt_acknowledge(),
            _ => 0,
        }
    }

    fn read_interrupt_acknowledge(&mut self) -> u64 {
        if self.irq_pending {
            let id = self.last_irq_id as u64;
            self.irq_pending = false;
            id
        } else {
            GIC_SPURIOUS_INTERRUPT
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mmfr1_does_not_advertise_unimplemented_hafdbs() {
        let mut sys = SystemRegisters::default();
        let mmfr1 = sys.read_sys_reg(SYSREG_ID_AA64MMFR1_EL1, 1);
        assert_eq!(mmfr1 & 0xF, 0);
    }
}
