use crate::constants::*;

// Re-export constants under their traditional shorter EFI names for backward compat.
pub const EFI_MEM_BASE: u64 = EFI_REGION_BASE;
pub const EFI_MEM_SIZE: u64 = EFI_REGION_SIZE;

pub const EFI_HANDLE_ADDR: u64 = crate::constants::EFI_HANDLE_ADDR;
pub const EFI_ST_PTR_ADDR: u64 = EFI_SYSTEM_TABLE_PTR_ADDR;
pub const EFI_SYSTEM_TABLE: u64 = EFI_SYSTEM_TABLE_ADDR;
pub const EFI_RUNTIME_SERVICES: u64 = EFI_RUNTIME_SERVICES_ADDR;
pub const EFI_BOOT_SERVICES: u64 = EFI_BOOT_SERVICES_ADDR;
pub const EFI_SERVICE_TRAMPOLINES: u64 = EFI_TRAMPOLINES_ADDR;
pub const EFI_LARGE_CODE: u64 = EFI_LARGE_CODE_ADDR;

pub const LARGE_CODE_STRIDE: u64 = LARGE_CODE_BLOCK_SIZE;
pub const TRAMPOLINE_STRIDE: u64 = TRAMPOLINE_SLOT_SIZE;
pub const MAX_TRAMPOLINES: usize = crate::constants::MAX_TRAMPOLINES;

pub const EFI_SUCCESS: u64 = crate::constants::EFI_SUCCESS;

/// Returns true when `addr` falls inside the EFI firmware region.
pub fn is_efi_addr(addr: u64) -> bool {
    addr >= EFI_REGION_BASE && addr < EFI_REGION_END
}
