//! System register IDs, feature register values, timer, and PSTATE constants.
//!
//! ARM64 system registers are identified by a 15-bit number composed as:
//!   sysreg_id = (op0 << 14) | (op1 << 11) | (CRn << 7) | (CRm << 3) | op2

// ============================================================================
// INSTRUCTION ENCODING CONSTANTS (AArch64)
// ============================================================================

/// NOP (hint) — the universal "do nothing" instruction.
pub const INSTR_NOP: u32 = 0xD503_201F;

/// RET X30 — return from subroutine.
pub const INSTR_RET: u32 = 0xD65F_03C0;

/// ERET — return from exception.
pub const INSTR_ERET: u32 = 0xD69F_03E0;

/// Base encoding for MOVZ Xd, #imm16 (hw=0).
pub const MOVZ_BASE: u32 = 0xD280_0000;

/// Base encoding for MOVK Xd, #imm16 (hw=0).
pub const MOVK_BASE: u32 = 0xF280_0000;

// -- Exception vector offsets from VBAR_EL1 --

/// Synchronous exception at the current exception level.
pub const VBAR_SYNC_CURRENT_EL: u64 = 0x200;
/// IRQ exception at the current exception level.
pub const VBAR_IRQ_CURRENT_EL: u64 = 0x80;
/// SuperVisor Call (SVC) exception.
pub const VBAR_SYNC_LOWER_EL_AARCH64: u64 = 0x400;

// ============================================================================
// SYSTEM REGISTER IDS (MRS/MSR encoding)
// ============================================================================

/// MIDR_EL1 — Main ID Register.
pub const SYSREG_MIDR_EL1: u16 = 0x4000;
/// MPIDR_EL1 — Multiprocessor Affinity Register.
pub const SYSREG_MPIDR_EL1: u16 = 0x4005;
/// CurrentEL — read the current Exception Level.
pub const SYSREG_CURRENTEL: u16 = 0x4212;
/// ID_AA64PFR0_EL1 — Processor Feature Register 0.
pub const SYSREG_ID_AA64PFR0_EL1: u16 = 0x4020;
/// ID_AA64PFR1_EL1 — Processor Feature Register 1.
pub const SYSREG_ID_AA64PFR1_EL1: u16 = 0x4021;
/// ID_AA64PFR2_EL1 — Processor Feature Register 2.
pub const SYSREG_ID_AA64PFR2_EL1: u16 = 0x4028;
/// ID_AA64DFR0_EL1 — Debug Feature Register 0.
pub const SYSREG_ID_AA64DFR0_EL1: u16 = 0x4030;
/// ID_AA64DFR1_EL1 — Debug Feature Register 1.
pub const SYSREG_ID_AA64DFR1_EL1: u16 = 0x4031;
/// ID_AA64ISAR0_EL1 — Instruction Set Attribute Register 0.
pub const SYSREG_ID_AA64ISAR0_EL1: u16 = 0x4032;
/// ID_AA64ISAR1_EL1 — Instruction Set Attribute Register 1.
pub const SYSREG_ID_AA64ISAR1_EL1: u16 = 0x4033;
/// ID_AA64ISAR2_EL1 — Instruction Set Attribute Register 2.
pub const SYSREG_ID_AA64ISAR2_EL1: u16 = 0x4034;
/// ID_AA64MMFR0_EL1 — Memory Model Feature Register 0.
pub const SYSREG_ID_AA64MMFR0_EL1: u16 = 0x4038;
/// ID_AA64MMFR1_EL1 — Memory Model Feature Register 1.
pub const SYSREG_ID_AA64MMFR1_EL1: u16 = 0x4039;
/// ID_AA64MMFR2_EL1 — Memory Model Feature Register 2.
pub const SYSREG_ID_AA64MMFR2_EL1: u16 = 0x403A;
/// SCTLR_EL1 — System Control Register (EL1).
pub const SYSREG_SCTLR_EL1: u16 = 0x4080;
/// CPACR_EL1 — Architectural Feature Access Control Register.
pub const SYSREG_CPACR_EL1: u16 = 0x4082;
/// TTBR0_EL1 — Translation Table Base Register 0.
pub const SYSREG_TTBR0_EL1: u16 = 0x4100;
/// TTBR1_EL1 — Translation Table Base Register 1.
pub const SYSREG_TTBR1_EL1: u16 = 0x4101;
/// TCR_EL1 — Translation Control Register.
pub const SYSREG_TCR_EL1: u16 = 0x4102;
/// SPSR_EL1 — Saved Program Status Register (EL1).
pub const SYSREG_SPSR_EL1: u16 = 0x4200;
/// ELR_EL1 — Exception Link Register (EL1).
pub const SYSREG_ELR_EL1: u16 = 0x4201;
/// SP_EL0 — Stack Pointer (EL0).
pub const SYSREG_SP_EL0: u16 = 0x4208;
/// ESR_EL1 — Exception Syndrome Register (EL1).
pub const SYSREG_ESR_EL1: u16 = 0x4290;
/// FAR_EL1 — Fault Address Register (EL1).
pub const SYSREG_FAR_EL1: u16 = 0x4300;
/// MAIR_EL1 — Memory Attribute Indirection Register.
pub const SYSREG_MAIR_EL1: u16 = 0x4510;
/// VBAR_EL1 — Vector Base Address Register (EL1).
pub const SYSREG_VBAR_EL1: u16 = 0x4600;
/// TPIDR_EL1 — Thread/Process ID Register (EL1).
pub const SYSREG_TPIDR_EL1: u16 = 0x4684;
/// CTR_EL0 — Cache Type Register.
pub const SYSREG_CTR_EL0: u16 = 0x5801;
/// DCZID_EL0 — Data Cache Zero ID Register (EL0).
pub const SYSREG_DCZID_EL0: u16 = 0x5807;
/// CNTFRQ_EL0 — Counter-Timer Frequency Register.
pub const SYSREG_CNTFRQ_EL0: u16 = 0x5F00;
/// CNTPCT_EL0 — Physical Counter value.
pub const SYSREG_CNTPCT_EL0: u16 = 0x5F01;
/// CNTVCT_EL0 — Virtual Counter value.
pub const SYSREG_CNTVCT_EL0: u16 = 0x5F02;
/// TPIDR_EL0 — User Read/Write Thread ID Register.
pub const SYSREG_TPIDR_EL0: u16 = 0x5E82;
/// TPIDRRO_EL0 — User Read-Only Thread ID Register.
pub const SYSREG_TPIDRRO_EL0: u16 = 0x5E83;
/// CNTP_TVAL_EL0 — Physical Timer Value.
pub const SYSREG_CNTP_TVAL_EL0: u16 = 0x5F10;
/// CNTP_CTL_EL0 — Physical Timer Control.
pub const SYSREG_CNTP_CTL_EL0: u16 = 0x5F11;
/// CNTP_CVAL_EL0 — Physical Timer Compare Value.
pub const SYSREG_CNTP_CVAL_EL0: u16 = 0x5F12;
/// DAIF — Interrupt Mask bits.
pub const SYSREG_DAIF: u16 = 0x5A11;
/// SCR_EL3 — Secure Configuration Register (EL3).
pub const SYSREG_SCR_EL3: u16 = 0x7088;
/// SPSR_EL3 — Saved Program Status Register (EL3).
pub const SYSREG_SPSR_EL3: u16 = 0x7200;
/// ELR_EL3 — Exception Link Register (EL3).
pub const SYSREG_ELR_EL3: u16 = 0x7201;
/// HCR_EL2 — Hypervisor Configuration Register (EL2).
pub const SYSREG_HCR_EL2: u16 = 0x6088;
/// SPSR_EL2 — Saved Program Status Register (EL2).
pub const SYSREG_SPSR_EL2: u16 = 0x6200;
/// ELR_EL2 — Exception Link Register (EL2).
pub const SYSREG_ELR_EL2: u16 = 0x6201;

// -- GICv3 CPU Interface system registers --

/// ICC_PMR_EL1 — Interrupt Priority Mask Register.
pub const SYSREG_ICC_PMR_EL1: u16 = 0x4230;
/// ICC_CTLR_EL1 — Interrupt Controller Control Register.
pub const SYSREG_ICC_CTLR_EL1: u16 = 0x4234;
/// ICC_IAR1_EL1 — Interrupt Acknowledge Register 1.
pub const SYSREG_ICC_IAR1_EL1: u16 = 0x4660;
/// ICC_EOIR1_EL1 — End Of Interrupt Register 1.
pub const SYSREG_ICC_EOIR1_EL1: u16 = 0x4661;
/// ICC_SRE_EL1 — System Register Enable.
pub const SYSREG_ICC_SRE_EL1: u16 = 0x4665;

// ============================================================================
// FEATURE REGISTER VALUES
// ============================================================================

/// MIDR_EL1: ARM Cortex-A72 revision r0p3.
pub const MIDR_CORTEX_A72_R0P3: u64 = 0x410FD083;
/// MPIDR_EL1: single core, Aff0=0.
pub const MPIDR_SINGLE_CORE: u64 = 0x80000000;
/// ID_AA64PFR0_EL1: EL0/1/2/3 all support AArch64.
pub const ID_AA64PFR0_EL1_VAL: u64 = 0x0000_0000_0000_0011;
/// ID_AA64MMFR0_EL1: 4K + 64K granules, 48-bit physical address.
pub const ID_AA64MMFR0_EL1_VAL: u64 = 0x0000_0000_0000_1122;
/// ID_AA64ISAR0_EL1: AES + PMULL + SHA1 + SHA256 + CRC32.
pub const ID_AA64ISAR0_EL1_VAL: u64 = 0x0000_0000_0010_3106;
/// ID_AA64ISAR1_EL1: DotProduct + LRCPC + FCMA + JSCVT.
pub const ID_AA64ISAR1_EL1_VAL: u64 = 0x0000_0000_0000_0121;
/// ID_AA64DFR0_EL1: Debug v8, PMU v3.
pub const ID_AA64DFR0_EL1_VAL: u64 = 0x0000_0000_0010_3106;
/// CTR_EL0: Cache Type Register value.
pub const CTR_EL0_VAL: u64 = 0x8444_c004;
/// DCZID_EL0: DC ZVA block size = 16 bytes.
pub const DCZID_EL0_VAL: u64 = 0x0000_0000_0000_0010;
/// GICD_IIDR: GICv3 revision r0, ARM implementation.
pub const GICD_IIDR_VAL: u32 = 0x0201743B;

// ============================================================================
// GENERIC TIMER CONSTANTS
// ============================================================================

/// Default counter-timer frequency: 62.5 MHz (matches QEMU's virt machine).
pub const TIMER_FREQ_HZ: u64 = 62_500_000;
/// Timer IRQ ID (PPI 30 = Non-secure Physical Timer).
pub const TIMER_IRQ_ID: u32 = 30;
/// Spurious interrupt ID — returned when no interrupt is pending.
pub const GIC_SPURIOUS_INTERRUPT: u64 = 1023;

/// CNTP_CTL_EL0.ENABLE
pub const TIMER_CTL_ENABLE: u64 = 1;
/// CNTP_CTL_EL0.IMASK
pub const TIMER_CTL_IMASK: u64 = 1 << 1;
/// CNTP_CTL_EL0.ISTATUS
pub const TIMER_CTL_ISTATUS: u64 = 1 << 2;

// ============================================================================
// PROCESSOR STATE (PSTATE) LAYOUT
// ============================================================================

/// N (Negative) flag bit.
pub const PSTATE_N_BIT: u32 = 31;
/// Z (Zero) flag bit.
pub const PSTATE_Z_BIT: u32 = 30;
/// C (Carry) flag bit.
pub const PSTATE_C_BIT: u32 = 29;
/// V (Overflow) flag bit.
pub const PSTATE_V_BIT: u32 = 28;
/// PSTATE mask for NZCV flags.
pub const PSTATE_NZCV_MASK: u64 = 0xF000_0000;
/// Exception Level field shift.
pub const PSTATE_EL_SHIFT: u32 = 2;
/// EL mask (2 bits).
pub const PSTATE_EL_MASK: u64 = 3 << PSTATE_EL_SHIFT;
/// IRQ mask bit (I).
pub const PSTATE_I_BIT: u32 = 7;
/// SPSR exception-level return mode bits [9:6].
pub const SPSR_M_MASK: u64 = 0xF << 6;

// ============================================================================
// ARCHITECTURAL CONSTANTS
// ============================================================================

/// Number of general-purpose registers (X0–X30).
pub const NUM_GENERAL_REGISTERS: u8 = 31;
/// The zero register — reading X31 / W31 always returns 0.
pub const ZERO_REGISTER_INDEX: u8 = 31;
/// Stack Pointer register index.
pub const SP_REGISTER_INDEX: u8 = 31;
/// Link register (X30) — holds the return address for BL/BLR.
pub const LINK_REGISTER_INDEX: u8 = 30;
/// Maximum exception level.
pub const MAX_EL: u8 = 3;

// ============================================================================
// REGISTER WIDTH MASKS
// ============================================================================

/// Sign bit position for 64-bit operations.
pub const SIGN_BIT_64: u32 = 63;
/// Sign bit position for 32-bit operations.
pub const SIGN_BIT_32: u32 = 31;
/// Mask that isolates the lower 32 bits.
pub const WORD_MASK: u64 = 0xFFFF_FFFF;
