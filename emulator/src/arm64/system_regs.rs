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
        }
    }
}
