/// Memory-mapped EFI structures sit just above RAM.
pub const EFI_MEM_BASE: u64 = 0x8000_0000;
pub const EFI_MEM_SIZE: u64 = 0x1000_0000;

pub const EFI_HANDLE_ADDR: u64 = EFI_MEM_BASE + 0x0000;
pub const EFI_ST_PTR_ADDR: u64 = EFI_MEM_BASE + 0x0008;
pub const EFI_SYSTEM_TABLE: u64 = EFI_MEM_BASE + 0x1000;
pub const EFI_RUNTIME_SERVICES: u64 = EFI_MEM_BASE + 0x2000;
pub const EFI_BOOT_SERVICES: u64 = EFI_MEM_BASE + 0x3000;
pub const EFI_SERVICE_TRAMPOLINES: u64 = EFI_MEM_BASE + 0x4000;

/// Dedicated area for large trampolines that exceed TRAMPOLINE_STRIDE.
/// Placed at 0x8000_C000, well above the stride-based slot table (ends at 0x8000_C000).
pub const EFI_LARGE_CODE: u64 = EFI_MEM_BASE + 0xC000;
/// Each large code block is 512 bytes — plenty for complex services.
pub const LARGE_CODE_STRIDE: u64 = 512;

pub const TRAMPOLINE_STRIDE: u64 = 32;
pub const MAX_TRAMPOLINES: usize = 256;
pub const EFI_SUCCESS: u64 = 0;

pub fn is_efi_addr(addr: u64) -> bool {
    addr >= EFI_MEM_BASE && addr < EFI_MEM_BASE + EFI_MEM_SIZE
}
