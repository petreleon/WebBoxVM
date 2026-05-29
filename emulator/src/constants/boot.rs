//! Boot page table addresses, EFI trap addresses, PE/COFF, DTB, CPIO, and misc constants.

// ============================================================================
// BOOT PAGE TABLE ADDRESSES
// ============================================================================

/// TTBR1 Level-0 table base.
pub const BOOT_TTBR1_L0: u64 = 0x8010_0000;
/// TTBR1 Level-1 table base.
pub const BOOT_TTBR1_L1: u64 = 0x8010_1000;
/// TTBR1 Level-2 table base.
pub const BOOT_TTBR1_L2: u64 = 0x8010_2000;
/// TTBR1 Level-3 table base (first of 96 tables).
pub const BOOT_TTBR1_L3_BASE: u64 = 0x8010_3000;
/// Number of L3 tables pre-allocated for the kernel VA mapping.
pub const BOOT_TTBR1_L3_COUNT: usize = 96;

/// TTBR0 Level-0 table base (identity map).
pub const BOOT_TTBR0_L0: u64 = 0x8017_3000;
/// TTBR0 Level-1 table base.
pub const BOOT_TTBR0_L1: u64 = 0x8017_4000;

/// Number of 1 GiB blocks for the identity map.
pub const IDENTITY_MAP_BLOCKS: usize = 4;

// ============================================================================
// EFI STUB TRAP ADDRESSES
// ============================================================================

/// PC trap: CopyMem stub.
pub const EFI_TRAP_COPYMEM: u64 = 0x8000_CE00;
/// PC trap: SetMem stub.
pub const EFI_TRAP_SETMEM: u64 = 0x8000_D000;
/// PC trap: AllocatePages stub.
pub const EFI_TRAP_ALLOCPAGES: u64 = 0x8000_D200;
/// PC trap: FreePages stub.
pub const EFI_TRAP_FREEPAGES: u64 = 0x8000_D400;

// -- Cache invalidation loop fast-forwards --

/// Cache maintenance loop entry PC.
pub const CACHE_INV_LOOP_ENTRY: u64 = 0x400b6e80;
/// Cache maintenance loop exit PC.
pub const CACHE_INV_LOOP_EXIT: u64 = 0x400b6e90;
/// Instruction cache invalidate loop entry PC.
pub const I_CACHE_INV_LOOP_ENTRY: u64 = 0x400b6eb8;
/// Instruction cache invalidate loop exit PC.
pub const I_CACHE_INV_LOOP_EXIT: u64 = 0x400b6ec8;

// ============================================================================
// PE/COFF HEADER CONSTANTS
// ============================================================================

/// ARM64 Linux kernel magic: `"ARM\x64"`.
pub const ARM64_KERNEL_MAGIC: u32 = 0x644d5241;
/// PE32+ optional header magic (0x20B).
pub const PE32PLUS_MAGIC: u16 = 0x020B;
/// `PE\0\0` signature.
pub const PE_SIGNATURE: &[u8; 4] = b"PE\0\0";
/// Offset in the kernel Image where the PE signature is expected.
pub const KERNEL_PE_OFFSET: usize = 0x40;
/// Minimum size of a PE optional header.
pub const PE_OPT_HEADER_MIN_SIZE: usize = 112;

// ============================================================================
// DEVICE TREE BLOB (DTB) CONSTANTS
// ============================================================================

/// FDT magic number (big-endian 0xD00DFEED).
pub const FDT_MAGIC: u32 = 0xD00DFEED;
/// FDT version (17).
pub const FDT_VERSION: u32 = 17;
/// FDT last compatible version (16).
pub const FDT_LAST_COMP_VERSION: u32 = 16;

// FDT tokens (structure block element types)
pub const FDT_BEGIN_NODE: u32 = 0x0000_0001;
pub const FDT_END_NODE: u32 = 0x0000_0002;
pub const FDT_PROP: u32 = 0x0000_0003;
pub const FDT_END: u32 = 0x0000_0009;

// ============================================================================
// CPIO (INITRD) CONSTANTS
// ============================================================================

/// cpio newc format magic.
pub const CPIO_NEWC_MAGIC: &str = "070701";
/// cpio newc trailer file name.
pub const CPIO_TRAILER_NAME: &str = "TRAILER!!!";
/// Size of a cpio newc header in bytes (110).
pub const CPIO_HEADER_SIZE: usize = 110;

// ============================================================================
// MISC
// ============================================================================

/// Maximum history entries kept for instruction tracing.
pub const INSTR_HISTORY_SIZE: usize = 100;
/// Timeslice: number of instructions each core runs before yielding.
pub const ROUND_ROBIN_TIMESLICE: usize = 10_000;
