use super::timer::timer_cval_from_tval;
use super::*;

impl SystemRegisters {
    /// Write a system register by its 15-bit ID.
    pub fn write_sys_reg(&mut self, sysreg_id: u16, val: u64) {
        match sysreg_id {
            SYSREG_SP_EL0 => self.sp_el0 = val,
            SYSREG_SP_EL1 => self.sp_el1 = val,
            SYSREG_TPIDR_EL0 => self.tpidr_el0 = val,
            SYSREG_TPIDR_EL1 => self.tpidr_el1 = val,
            SYSREG_TPIDRRO_EL0 => self.tpidrro_el0 = val,
            SYSREG_SCTLR_EL1 => self.sctlr_el1 = val,
            SYSREG_TCR_EL1 => self.tcr_el1 = val,
            SYSREG_TTBR0_EL1 => self.ttbr0_el1 = val,
            SYSREG_TTBR1_EL1 => self.ttbr1_el1 = val,
            SYSREG_MAIR_EL1 => self.mair_el1 = val,
            SYSREG_VBAR_EL1 => self.vbar_el1 = val,
            SYSREG_SPSR_EL1 => self.spsr_el1 = val,
            SYSREG_ELR_EL1 => self.elr_el1 = val,
            SYSREG_ESR_EL1 => self.esr_el1 = val,
            SYSREG_FAR_EL1 => self.far_el1 = val,
            SYSREG_CPACR_EL1 => self.cpacr_el1 = val,
            SYSREG_FPCR => self.fpcr = val,
            SYSREG_FPSR => self.fpsr = val,
            SYSREG_CNTFRQ_EL0 => self.cntfrq_el0 = val,
            SYSREG_CNTKCTL_EL1 => self.cntkctl_el1 = val,
            SYSREG_ICC_PMR_EL1 => self.icc_pmr_el1 = val,
            SYSREG_ICC_CTLR_EL1 => self.icc_ctlr_el1 = val,
            SYSREG_ICC_SRE_EL1 => self.icc_sre_el1 = val,
            SYSREG_ICC_EOIR1_EL1 => self.end_interrupt(),
            SYSREG_CNTP_TVAL_EL0 => self.write_cntp_tval(val),
            SYSREG_CNTP_CTL_EL0 => self.cntp_ctl_el0 = val & (TIMER_CTL_ENABLE | TIMER_CTL_IMASK),
            SYSREG_CNTP_CVAL_EL0 => self.cntp_cval_el0 = val,
            SYSREG_CNTV_TVAL_EL0 => self.write_cntv_tval(val),
            SYSREG_CNTV_CTL_EL0 => self.cntv_ctl_el0 = val & (TIMER_CTL_ENABLE | TIMER_CTL_IMASK),
            SYSREG_CNTV_CVAL_EL0 => self.cntv_cval_el0 = val,
            SYSREG_DAIF => {}
            SYSREG_SCR_EL3 => self.scr_el3 = val,
            SYSREG_SPSR_EL3 => self.spsr_el3 = val,
            SYSREG_ELR_EL3 => self.elr_el3 = val,
            SYSREG_HCR_EL2 => self.hcr_el2 = val,
            SYSREG_SPSR_EL2 => self.spsr_el2 = val,
            SYSREG_ELR_EL2 => self.elr_el2 = val,
            _ => {}
        }
    }

    fn end_interrupt(&mut self) {
        self.irq_pending = false;
        self.last_irq_id = GIC_SPURIOUS_INTERRUPT as u32;
    }

    fn write_cntp_tval(&mut self, val: u64) {
        self.cntp_tval_el0 = val;
        self.cntp_cval_el0 = timer_cval_from_tval(self.cycle_count, val);
    }

    fn write_cntv_tval(&mut self, val: u64) {
        self.cntv_tval_el0 = val;
        self.cntv_cval_el0 = timer_cval_from_tval(self.cycle_count, val);
    }
}
