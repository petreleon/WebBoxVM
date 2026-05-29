//! Large EFI service trampolines (> 32 bytes of code).

use crate::constants::*;
use crate::efi::encode::{movz_x, movk_x};
use crate::efi::protocols::LOADED_IMAGE_GUID_LO;
use super::{encode_ret, encode_mov64};

/// Fixed address for the page bump-allocator head.
pub(super) const EFI_PAGE_ALLOC_HEAD: u64 = EFI_REGION_BASE + 0xFF10;

/// Build the GetMemoryMap trampoline.
///
/// Reports a single EFI_MEMORY_DESCRIPTOR covering the entire RAM region.
/// If the caller's buffer is too small, returns EFI_BUFFER_TOO_SMALL.
pub(super) fn build_get_memory_map_trampoline() -> Vec<u32> {
    let mut v = Vec::new();

    // X6 = *MemoryMapSize (current buffer size)
    v.push(0xf9400006); // LDR X6, [X0]

    // CMP X6, #48  (SUBS XZR, X6, #48) — is the buffer big enough?
    v.push(0xF100C0DF);

    // B.HS label_fill — if X6 >= 48, skip to "fill" logic
    v.push(0x54000002); // placeholder, patched below

    // ── label_too_small: buffer isn't big enough ──
    encode_mov64(&mut v, 5, EFI_MEMORY_DESC_SIZE);   // *MemoryMapSize = 48
    v.push(0xf9000005); // STR X5, [X0]
    v.push(0xf9000065); // *DescriptorSize = 48
    encode_mov64(&mut v, 5, EFI_MEMORY_DESC_VERSION);
    v.push(0xb9000085); // *DescriptorVersion = 1

    encode_mov64(&mut v, 0, EFI_BUFFER_TOO_SMALL);
    v.push(encode_ret());

    let too_small_len = v.len() as i32;

    // ── label_fill: write one EFI_MEMORY_DESCRIPTOR at [X1] ──
    // Type = EfiConventionalMemory (7)
    encode_mov64(&mut v, 5, EFI_CONVENTIONAL_MEMORY_TYPE);
    v.push(0xb9000025); // STR W5, [X1]        // Type
    v.push(0xb900043f); // STR WZR, [X1, #4]  // Pad = 0
    encode_mov64(&mut v, 5, RAM_BASE);
    v.push(0xf9000425); // STR X5, [X1, #8]   // PhysicalStart
    v.push(0xf900083f); // STR XZR, [X1, #16] // VirtualStart = 0
    encode_mov64(&mut v, 5, 0x40000u64);      // NumberOfPages = 1 GiB / 4 KiB
    v.push(0xf9000c25); // STR X5, [X1, #24]  // NumberOfPages
    encode_mov64(&mut v, 5, EFI_MEMORY_WB);
    v.push(0xf9001025); // STR X5, [X1, #32]  // Attribute
    v.push(0xf900143f); // STR XZR, [X1, #40] // Pad2 = 0

    // Set outputs
    encode_mov64(&mut v, 5, EFI_MEMORY_DESC_SIZE);
    v.push(0xf9000005); // *MemoryMapSize = 48
    encode_mov64(&mut v, 5, EFI_MEMORY_MAP_KEY);
    v.push(0xf9000045); // *MapKey = 17
    encode_mov64(&mut v, 5, EFI_MEMORY_DESC_SIZE);
    v.push(0xf9000065); // *DescriptorSize = 48
    encode_mov64(&mut v, 5, EFI_MEMORY_DESC_VERSION);
    v.push(0xb9000085); // *DescriptorVersion = 1
    v.push(movz_x(0, 0)); // EFI_SUCCESS
    v.push(encode_ret());

    // Patch the B.HS branch target
    let branch_offset = (too_small_len - 2) as u32;
    let bcond_hs = 0x54000002u32 | ((branch_offset & 0x7FFFF) << 5);
    v[2] = bcond_hs;

    v
}

/// Build the HandleProtocol / OpenProtocol trampoline.
///
/// Checks if the requested GUID matches EFI_LOADED_IMAGE_PROTOCOL.
/// If yes: *Interface = LIP address, returns EFI_SUCCESS.
/// If no:  *Interface = NULL, returns EFI_NOT_FOUND.
pub(super) fn build_handle_protocol_trampoline(lip_addr: u64) -> Vec<u32> {
    let mut v = Vec::new();

    // LDR X4, [X1] — load first 8 bytes of GUID
    v.push(0xf9400024);
    // Build expected GUID low bits into X3
    encode_mov64(&mut v, 3, LOADED_IMAGE_GUID_LO);
    // SUBS X4, X4, X3 — compare
    v.push(0xEB030084);
    // CBNZ X4, label_not_found — placeholder, patched below
    let cbnz_idx = v.len();
    v.push(0xB5000004);

    // ── GUID matches: return LIP ──
    encode_mov64(&mut v, 3, lip_addr);
    v.push(0xf9000043); // STR X3, [X2]
    v.push(movz_x(0, 0)); // EFI_SUCCESS
    v.push(encode_ret());

    let not_found_idx = v.len();

    // ── GUID doesn't match ──
    v.push(0xF900005F); // STR XZR, [X2]  // *Interface = NULL
    encode_mov64(&mut v, 0, EFI_NOT_FOUND);
    v.push(encode_ret());

    // Patch CBNZ X4: offset to not_found label
    let offset = (not_found_idx as i32 - cbnz_idx as i32) as u32;
    v[cbnz_idx] = 0xB5000004u32 | ((offset & 0x7FFFF) << 5);

    v
}

/// LocateProtocol trampoline: return EFI_NOT_FOUND, *Interface = NULL.
pub(super) fn build_locate_protocol_trampoline() -> Vec<u32> {
    let mut v = Vec::new();
    v.push(0xB4000042); // CBZ X2, skip_store — guard null pointer
    v.push(0xF900005F); // STR XZR, [X2] — *Interface = NULL
    encode_mov64(&mut v, 0, EFI_NOT_FOUND);
    v.push(encode_ret());
    v
}

/// Build the AllocatePages trampoline.
///
/// Signature: AllocatePages(Type=X0, MemoryType=X1, Pages=X2, *Memory=X3)
///
/// Loads the current bump head from EFI_PAGE_ALLOC_HEAD, rounds up to page
/// boundary, bumps by Pages * 4096, stores the allocated address into *Memory,
/// and returns EFI_SUCCESS.
pub(super) fn build_allocate_pages_trampoline(head_ptr: u64, _init_base: u64) -> Vec<u32> {
    let mut v = Vec::new();

    // Load head pointer address into X4
    encode_mov64(&mut v, 4, head_ptr);

    // LDR X5, [X4]       — read current bump head
    v.push(0xF9400085);

    // ADD X6, X5, #4095   — round up to page boundary
    v.push(0x910FFCA6);

    // AND X6, X6, #0xFFFFFFFFFFFFF000  — clear lower 12 bits (page align)
    v.push(0x9272D0C6);

    // LSL X7, X2, #12    — Pages * 4096 (page size)
    v.push(0xD37CEC47);

    // ADD X7, X6, X7     — new head = alloc + size
    v.push(0x8B0700C7);

    // STR X7, [X4]       — update bump head
    v.push(0xF9000087);

    // STR X6, [X3]       — write allocated address to *Memory
    v.push(0xF9000066);

    // MOV X0, #0         — EFI_SUCCESS
    v.push(movz_x(0, 0));

    // RET
    v.push(encode_ret());

    v
}
