//! WebBoxVM Constants — ALL magic numbers and hardware definitions documented.
//!
//! This module is the single source of truth for every address, register ID,
//! bitmask, and architectural constant used across the emulator.  If you see a
//! bare hex number somewhere else in the codebase, please move it here.
//!
//! ============================================================================
//! HOW ARM64 PHYSICAL MEMORY IS ORGANISED
//! ============================================================================
//!
//! The emulator presents a flat 64-bit physical address space.  For the Linux
//! boot to work we reserve three disjoint regions:
//!
//!   0x0000_0000  ─►  0x3FFF_FFFF   Low region (1 GiB)
//!     ├─ 0x0800_0000  GICv3 interrupt controller
//!     └─ 0x0900_0000  PL011 UART (serial console)
//!
//!   0x4000_0000  ─►  0x7FFF_FFFF   RAM region (1 GiB)
//!     ├─ 0x4008_0000  Kernel image loaded here
//!     ├─ 0x43A0_A000  EFI pool allocator starts here
//!     ├─ 0x43EF_E000  EFI-stub return trampoline (a single RET instruction)
//!     ├─ 0x43F0_0000  Boot stack pointer (SP)
//!     ├─ 0x4400_0000  Initrd (initial RAM disk) loaded here
//!     └─ 0x4700_0000  Device Tree Blob (DTB) address
//!
//!   0x8000_0000  ─►  0x8FFF_FFFF   EFI region (256 MiB)
//!     ├─ 0x8000_0000  EFI image handle
//!     ├─ 0x8000_1000  EFI System Table
//!     ├─ 0x8000_8000  Loaded Image Protocol (LIP)
//!     └─ 0x8010_0000  Page tables (MMU translation tables)

mod layout;
mod efi;
mod sysreg;
mod mmu;
mod boot;

pub use layout::*;
pub use efi::*;
pub use sysreg::*;
pub use mmu::*;
pub use boot::*;
