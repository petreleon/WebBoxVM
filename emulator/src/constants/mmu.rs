//! MMU, page table, and TLB constants.

// ============================================================================
// MMU / PAGE TABLE CONSTANTS
// ============================================================================

/// Virtual address width (48-bit as seen by ARMv8.0-A).
pub const VA_BITS_MAX: u8 = 48;

/// VA split point — kernel VAs start at 0xFFFF_8000_0000_0000.
pub const KERNEL_VA_BASE: u64 = 0xFFFF_8000_0000_0000;

/// Low 32-bit mask used to extract device-identity offset from kernel VA.
pub const VA_LOW32_MASK: u64 = 0xFFFF_FFFF;

/// Fixmap mask: low 21 bits of the VA identify the device.
pub const FIXMAP_LOW_MASK: u64 = 0x001F_FFFF;

/// Number of entries in a page table (512 per level, 9 bits).
pub const PT_ENTRIES: u64 = 512;

/// Shift for page-table index at each level.
pub const PT_L0_SHIFT: u64 = 39;
pub const PT_L1_SHIFT: u64 = 30;
pub const PT_L2_SHIFT: u64 = 21;
pub const PT_L3_SHIFT: u64 = 12;

/// Size of each level-n block in bytes.
pub const L0_BLOCK_SIZE: u64 = 1 << PT_L0_SHIFT;

/// Page table descriptor type bits.
pub const DESC_VALID: u64 = 0b11;
/// Descriptor[1:0] = 0b01 means block/page.
pub const DESC_BLOCK: u64 = 0b01;
/// Descriptor[1:0] = 0b11 means table pointer.
pub const DESC_TABLE: u64 = 0b11;
/// Mask to extract the output address from a descriptor.
pub const DESC_ADDR_MASK: u64 = 0x0000_FFFF_FFFF_F000;

/// TCR_EL1.T0SZ / T1SZ field position and mask.
pub const TCR_T0SZ_SHIFT: u64 = 0;
pub const TCR_T0SZ_MASK: u64 = 0x3F;
pub const TCR_T1SZ_SHIFT: u64 = 16;
pub const TCR_T1SZ_MASK: u64 = 0x3F;

/// MAIR_EL1 default value: outer/inner write-back cacheable.
pub const MAIR_EL1_DEFAULT: u64 = 0xFF;

/// SCTLR_EL1.M — MMU enable bit.
pub const SCTLR_EL1_M_BIT: u64 = 1;
/// Alias for the MMU enable bit.
pub const SCTLR_MMU_ENABLE: u64 = 1;

/// Page table access flag (bit 10).
pub const DESC_AF_BIT: u64 = 1 << 10;

// ============================================================================
// TLB (TRANSLATION LOOKASIDE BUFFER)
// ============================================================================

/// Number of TLB entries (2048, direct-mapped by VA bits [23:12]).
pub const TLB_ENTRIES: usize = 2048;
/// TLB index mask (11 bits = 2048 entries).
pub const TLB_INDEX_MASK: u64 = 0x7FF;

// ============================================================================
// INSTRUCTION SIZE & FETCH
// ============================================================================

/// Every AArch64 instruction is exactly 4 bytes.
pub const INSTRUCTION_SIZE: u64 = 4;
/// Instructions per 4 KiB page (used for decode cache).
pub const INSTRUCTIONS_PER_PAGE: usize = 1024;
