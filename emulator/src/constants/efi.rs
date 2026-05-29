//! EFI firmware region layout and service-table constants.

// ============================================================================
// EFI FIRMWARE REGION  (0x8000_0000 – 0x8FFF_FFFF)
// ============================================================================

/// Base of the 256 MiB EFI region.
pub const EFI_REGION_BASE: u64 = 0x8000_0000;
/// Size of the EFI region.
pub const EFI_REGION_SIZE: u64 = 0x1000_0000;
/// End of the EFI region (exclusive).
pub const EFI_REGION_END: u64 = EFI_REGION_BASE + EFI_REGION_SIZE;

// -- EFI data structures (offset from EFI_REGION_BASE) --

/// Address of the EFI image handle.
pub const EFI_HANDLE_ADDR: u64 = EFI_REGION_BASE + 0x0000;
/// Address where the system-table pointer is stored.
pub const EFI_SYSTEM_TABLE_PTR_ADDR: u64 = EFI_REGION_BASE + 0x0008;
/// Base of the EFI System Table structure (656 bytes).
pub const EFI_SYSTEM_TABLE_ADDR: u64 = EFI_REGION_BASE + 0x1000;
/// Base of the EFI Runtime Services table.
pub const EFI_RUNTIME_SERVICES_ADDR: u64 = EFI_REGION_BASE + 0x2000;
/// Base of the EFI Boot Services table.
pub const EFI_BOOT_SERVICES_ADDR: u64 = EFI_REGION_BASE + 0x3000;
/// Base of the trampoline code area.
pub const EFI_TRAMPOLINES_ADDR: u64 = EFI_REGION_BASE + 0x4000;
/// Area for large service trampolines (> 32 bytes).
pub const EFI_LARGE_CODE_ADDR: u64 = EFI_REGION_BASE + 0xC000;

/// Each standard trampoline slot is 32 bytes.
pub const TRAMPOLINE_SLOT_SIZE: u64 = 32;
/// Number of standard trampoline slots available.
pub const MAX_TRAMPOLINES: usize = 256;
/// Each large-code trampoline block is 512 bytes.
pub const LARGE_CODE_BLOCK_SIZE: u64 = 512;

/// Base of the ConsoleOut / StdErr protocol stubs.
pub const CONSOLE_OUT_STRUCT_ADDR: u64 = EFI_REGION_BASE + 0x6000;
/// Base of the ConsoleOut handle area.
pub const CONSOLE_OUT_HANDLE_ADDR: u64 = EFI_REGION_BASE + 0x5000;
/// Address of the FDT configuration table entry.
pub const EFI_CONFIG_TABLE_ADDR: u64 = EFI_REGION_BASE + 0x9000;

/// EFI image base/size store for later use.
pub const EFI_IMAGE_INFO_ADDR: u64 = EFI_REGION_BASE + 0xFF00;

// -- EFI_LOADED_IMAGE_PROTOCOL (LIP) --

/// Offset of the Loaded Image Protocol structure.
pub const LIP_STRUCT_ADDR: u64 = EFI_REGION_BASE + 0x8000;
/// The GUID that identifies EFI_LOADED_IMAGE_PROTOCOL.
pub const LIP_GUID: u128 = 0x5B1B31A1_9562_11D2_8E3F_00A0C969723B;
/// First 8 bytes of the LIP GUID in little-endian host order.
pub const LIP_GUID_LO: u64 = 0x11D2_9562_5B1B_31A1;

// ============================================================================
// EFI TABLE LAYOUT OFFSETS & SIGNATURES
// ============================================================================

/// EFI System Table signature magic: `"IBI SYST"` in ASCII LE.
pub const EFI_ST_SIGNATURE: u64 = 0x5453_5953_2049_4249;
/// EFI specification revision (2.6 = 0x0002_001E).
pub const EFI_ST_REVISION: u32 = 0x0002_001E;
/// Header size of the EFI System Table.
pub const EFI_ST_HEADER_SIZE: u32 = 0x78;

// -- Boot Services table offsets (0x18..0x170, every 8 bytes, 44 entries) --

pub const BS_RAISE_TPL_OFFSET: u64 = 0x18;
pub const BS_ALLOCATE_PAGES_OFFSET: u64 = 0x28;
pub const BS_FREE_PAGES_OFFSET: u64 = 0x30;
pub const BS_GET_MEMORY_MAP_OFFSET: u64 = 0x38;
pub const BS_ALLOCATE_POOL_OFFSET: u64 = 0x40;
pub const BS_HANDLE_PROTOCOL_OFFSET: u64 = 0x98;
pub const BS_OPEN_PROTOCOL_OFFSET: u64 = 0x118;
pub const BS_LOCATE_PROTOCOL_OFFSET: u64 = 0x140;
pub const BS_COPY_MEM_OFFSET: u64 = 0x160;
pub const BS_SET_MEM_OFFSET: u64 = 0x168;

/// EFI_SUCCESS.
pub const EFI_SUCCESS: u64 = 0;
/// EFI_NOT_FOUND.
pub const EFI_NOT_FOUND: u64 = 0x8000_0000_0000_000E;
/// EFI_BUFFER_TOO_SMALL.
pub const EFI_BUFFER_TOO_SMALL: u64 = 0x8000_0000_0000_0005;

/// Address of the boot-services vtable pointer for dereferencing.
pub const BOOT_SERVICES_VPTR_ADDR: u64 = 0x60;

// ============================================================================
// EFI MISCELLANEOUS
// ============================================================================

/// Maximum size for an EFI service copy/fill operation (safety bound).
pub const EFI_MAX_COPY_SIZE: u64 = 0x0400_0000;

/// Number of bytes per EFI_MEMORY_DESCRIPTOR (v1 = 48 bytes).
pub const EFI_MEMORY_DESC_SIZE: u64 = 48;

/// EFI_MEMORY_DESCRIPTOR count for GetMemoryMap response.
pub const EFI_MEMORY_DESC_COUNT: u64 = 1;

/// EFI conventional memory type.
pub const EFI_CONVENTIONAL_MEMORY_TYPE: u64 = 7;

/// EFI memory attribute: Write-Back cacheable.
pub const EFI_MEMORY_WB: u64 = 0xF;

/// EFI memory map key (arbitrary, must be non-zero).
pub const EFI_MEMORY_MAP_KEY: u64 = 17;

/// EFI descriptor version.
pub const EFI_MEMORY_DESC_VERSION: u64 = 1;

/// EFI LIP revision (0x1000).
pub const LIP_REVISION: u64 = 0x1000;
