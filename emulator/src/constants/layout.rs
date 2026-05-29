//! Memory layout constants — every address the emulator knows about.

// ============================================================================
// ADDRESS SPACE SIZES
// ============================================================================

/// Width of the physical address space in bits.
pub const PHYSICAL_ADDRESS_BITS: u8 = 36;

/// Size of a 4 KiB page — the smallest unit the MMU maps.
pub const PAGE_SIZE: u64 = 4096;
/// log2 of PAGE_SIZE (used for VA→PA offset extraction).
pub const PAGE_SHIFT: u64 = 12;
/// Mask that isolates the in-page offset (lower 12 bits).
pub const PAGE_OFFSET_MASK: u64 = PAGE_SIZE - 1;

/// Size of a 1 GiB block (Level-1 page table block).
pub const L1_BLOCK_SIZE: u64 = 0x4000_0000;
/// Alias for L1_BLOCK_SIZE.
pub const L1_BLOCK_SIZE_BYTES: u64 = L1_BLOCK_SIZE;

/// Size of a 2 MiB block (Level-2 page table block).
pub const L2_BLOCK_SIZE: u64 = 0x20_0000;
/// Alias for L2_BLOCK_SIZE.
pub const L2_BLOCK_SIZE_BYTES: u64 = L2_BLOCK_SIZE;

// ============================================================================
// LOW MEMORY REGION  (0x0000_0000 – 0x3FFF_FFFF)
// ============================================================================

/// Start of the low memory region — first byte of addressable memory.
pub const LOW_REGION_BASE: u64 = 0x0000_0000;
/// Size of the low memory region (1 GiB).
pub const LOW_REGION_SIZE: u64 = 0x4000_0000;
/// End of the low memory region (exclusive).
pub const LOW_REGION_END: u64 = LOW_REGION_BASE + LOW_REGION_SIZE;

// -- MMIO devices inside the low region --

/// Base address of the GICv3 Distributor (GICD) registers.
pub const GICD_BASE: u64 = 0x0800_0000;
/// Range size of the GICD register window.
pub const GICD_SIZE: u64 = 0x1_0000;

/// Base address of the GICv3 Redistributor (GICR) registers.
pub const GICR_BASE: u64 = 0x080A_0000;
/// Range size of the GICR register window.
pub const GICR_SIZE: u64 = 0xF6_0000;

/// Combined GIC area — covers both distributor and redistributor.
pub const GIC_MMIO_BASE: u64 = 0x0800_0000;
pub const GIC_MMIO_END: u64 = 0x0900_0000;

/// Base address of the PL011 UART (serial console).
pub const UART_BASE: u64 = 0x0900_0000;
/// Range size of the UART register window.
pub const UART_SIZE: u64 = 0x1000;
/// End of the UART MMIO region (exclusive).
pub const UART_END: u64 = UART_BASE + UART_SIZE;

/// PL011 UART register offsets (from base address).
pub const UART_DR_OFFSET: u64 = 0x00;
pub const UART_RSR_OFFSET: u64 = 0x04;
pub const UART_FR_OFFSET: u64 = 0x18;
pub const UART_IBRD_OFFSET: u64 = 0x24;
pub const UART_FBRD_OFFSET: u64 = 0x28;
pub const UART_LCR_H_OFFSET: u64 = 0x2C;
pub const UART_CR_OFFSET: u64 = 0x30;
pub const UART_IFLS_OFFSET: u64 = 0x34;
pub const UART_IMSC_OFFSET: u64 = 0x38;
pub const UART_RIS_OFFSET: u64 = 0x3C;
pub const UART_MIS_OFFSET: u64 = 0x40;
pub const UART_ICR_OFFSET: u64 = 0x44;
pub const UART_DMACR_OFFSET: u64 = 0x48;

// ============================================================================
// RAM REGION  (0x4000_0000 – 0x7FFF_FFFF)
// ============================================================================

/// Base of the physical RAM region.
pub const RAM_BASE: u64 = 0x4000_0000;
/// Amount of RAM available to the guest (1 GiB).
pub const RAM_SIZE: u64 = 0x4000_0000;
/// End of RAM (exclusive).
pub const RAM_END: u64 = RAM_BASE + RAM_SIZE;

/// Physical address where the kernel Image is loaded.
pub const KERNEL_LOAD_ADDR: u64 = RAM_BASE; // 0x4000_0000 (2MB-aligned, no UEFI Page Zero)

/// PE/EFI entry-point RVA for the custom-built kernel Image (6.6.70, CONFIG_RELOCATABLE=y).
pub const KERNEL_PE_ENTRY_OFFSET: u64 = 0x19ef668;

/// Kernel text-entry virtual address (VA) after the EFI stub exits.
pub const KERNEL_TEXT_VIRTUAL_ENTRY: u64 = 0xffff800080080000;

// -- EFI pool allocator (inside RAM) --

/// Start of the EFI pool for AllocatePool / AllocatePages requests.
pub const EFI_POOL_BASE: u64 = 0x43A0_A000;

/// Address of the bump-head pointer that tracks the next free pool byte.
pub const EFI_POOL_HEAD_PTR: u64 = 0x8000_FFF8;

// -- Boot trampoline inside RAM --

/// A single `RET` instruction planted here so the EFI stub can return.
pub const RETURN_TRAMPOLINE_ADDR: u64 = 0x43EF_E000;
/// Initial stack pointer loaded before jumping into the EFI stub.
pub const BOOT_STACK_POINTER: u64 = 0x43F0_0000;

// -- Initrd (initial RAM disk) --

/// Start address where the cpio initrd archive is loaded.
pub const INITRD_BASE: u64 = 0x4400_0000;

// -- Device Tree Blob (DTB) --

/// Address where the DTB is placed before handing off to the kernel.
pub const DTB_BASE: u64 = 0x4700_0000;

// -- Bump allocator for EFI AllocatePages --

/// Base address of the bump page allocator, grows upward.
pub const PAGE_ALLOCATOR_BASE: u64 = 0x4800_0000;
