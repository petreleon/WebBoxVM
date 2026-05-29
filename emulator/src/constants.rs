//! WebBoxVM Constants — ALL magic numbers and hardware defintions documented.
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

/// Size of a 2 MiB block (Level-2 page table block).
pub const L2_BLOCK_SIZE: u64 = 0x20_0000;

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
/// The GIC is the ARM interrupt controller; this is where the kernel
/// configures which interrupts are enabled and their priorities.
pub const GICD_BASE: u64 = 0x0800_0000;
/// Range size of the GICD register window.
pub const GICD_SIZE: u64 = 0x1_0000;

/// Base address of the GICv3 Redistributor (GICR) registers.
/// One redistributor per CPU core; handles per-core interrupt routing.
pub const GICR_BASE: u64 = 0x080A_0000;
/// Range size of the GICR register window.
pub const GICR_SIZE: u64 = 0xF6_0000;

/// Combined GIC area — covers both distributor and redistributor.
pub const GIC_MMIO_BASE: u64 = 0x0800_0000;
pub const GIC_MMIO_END: u64 = 0x0900_0000;

/// Base address of the PL011 UART (serial console).
/// This is where Linux writes its early boot messages via `earlycon=pl011`.
pub const UART_BASE: u64 = 0x0900_0000;
/// Range size of the UART register window.
pub const UART_SIZE: u64 = 0x1000;
/// End of the UART MMIO region (exclusive).
pub const UART_END: u64 = UART_BASE + UART_SIZE;

/// PL011 UART register offsets (from base address).
/// See the PL011 Technical Reference Manual and Linux driver at
/// drivers/tty/serial/amba-pl011.c for full register semantics.
pub const UART_DR_OFFSET: u64 = 0x00;     // Data Register (R/W)
pub const UART_RSR_OFFSET: u64 = 0x04;    // Receive Status / Error Clear
pub const UART_FR_OFFSET: u64 = 0x18;     // Flag Register (R)
pub const UART_IBRD_OFFSET: u64 = 0x24;   // Integer Baud Rate Divisor
pub const UART_FBRD_OFFSET: u64 = 0x28;   // Fractional Baud Rate Divisor
pub const UART_LCR_H_OFFSET: u64 = 0x2C;  // Line Control Register (high)
pub const UART_CR_OFFSET: u64 = 0x30;     // Control Register (R/W)
pub const UART_IFLS_OFFSET: u64 = 0x34;   // Interrupt FIFO Level Select
pub const UART_IMSC_OFFSET: u64 = 0x38;   // Interrupt Mask Set/Clear
pub const UART_RIS_OFFSET: u64 = 0x3C;    // Raw Interrupt Status (R)
pub const UART_MIS_OFFSET: u64 = 0x40;    // Masked Interrupt Status (R)
pub const UART_ICR_OFFSET: u64 = 0x44;    // Interrupt Clear (W)
pub const UART_DMACR_OFFSET: u64 = 0x48;  // DMA Control Register

// ============================================================================
// RAM REGION  (0x4000_0000 – 0x7FFF_FFFF)
// ============================================================================

/// Base of the physical RAM region — where the kernel and all runtime data live.
pub const RAM_BASE: u64 = 0x4000_0000;
/// Amount of RAM available to the guest (1 GiB).
pub const RAM_SIZE: u64 = 0x4000_0000;
/// End of RAM (exclusive).
pub const RAM_END: u64 = RAM_BASE + RAM_SIZE;

/// Physical address where the compressed kernel Image is loaded.
/// Offset 0x8_0000 from RAM_BASE — keeps the first 512 KiB free for
/// UEFI Page Zero (the kernel's own relocation dance).
pub const KERNEL_LOAD_ADDR: u64 = RAM_BASE + 0x8_0000; // 0x4008_0000

/// PE/EFI entry-point RVA for the custom-built kernel Image (6.6.70, CONFIG_RELOCATABLE=y).
pub const KERNEL_PE_ENTRY_OFFSET: u64 = 0x19ef668;

/// Kernel text-entry virtual address (VA) after the EFI stub exits.
/// This is the arch/arm64/kernel/head.S entry point in the kernel's VA space.
pub const KERNEL_TEXT_VIRTUAL_ENTRY: u64 = 0xffff800080080000;

// -- EFI pool allocator (inside RAM) --

/// Start of the EFI pool for AllocatePool / AllocatePages requests.
/// Must sit comfortably below the boot stack at 0x43F0_0000.
pub const EFI_POOL_BASE: u64 = 0x43A0_A000;

/// Address of the bump-head pointer that tracks the next free pool byte.
/// Stored inside the EFI scratch area for convenience.
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

/// Address of the EFI image handle (points to the loaded image protocol).
pub const EFI_HANDLE_ADDR: u64 = EFI_REGION_BASE + 0x0000;
/// Address where the system-table pointer is stored.
pub const EFI_SYSTEM_TABLE_PTR_ADDR: u64 = EFI_REGION_BASE + 0x0008;
/// Base of the EFI System Table structure (656 bytes).
pub const EFI_SYSTEM_TABLE_ADDR: u64 = EFI_REGION_BASE + 0x1000;
/// Base of the EFI Runtime Services table.
pub const EFI_RUNTIME_SERVICES_ADDR: u64 = EFI_REGION_BASE + 0x2000;
/// Base of the EFI Boot Services table.
pub const EFI_BOOT_SERVICES_ADDR: u64 = EFI_REGION_BASE + 0x3000;
/// Base of the trampoline code area — small ARM64 code snippets for each service.
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
/// Stored as 128-bit: 5B1B31A1-9562-11D2-8E3F-00A0C969723B
pub const LIP_GUID: u128 = 0x5B1B31A1_9562_11D2_8E3F_00A0C969723B;
/// First 8 bytes of the LIP GUID in little-endian host order
/// (Data1 LE u32 + Data2 LE u16 + Data3 LE u16 = 0x11D2_9562_5B1B_31A1).
pub const LIP_GUID_LO: u64 = 0x11D2_9562_5B1B_31A1;

// ============================================================================
// EFI TABLE LAYOUT OFFSETS & SIGNATURES
// ============================================================================

/// EFI System Table signature magic value: `"IBI SYST"` in ASCII LE.
pub const EFI_ST_SIGNATURE: u64 = 0x5453_5953_2049_4249;
/// EFI specification revision (2.6 = 0x0002_001E).
pub const EFI_ST_REVISION: u32 = 0x0002_001E;
/// Header size of the EFI System Table.
pub const EFI_ST_HEADER_SIZE: u32 = 0x78;

// -- Boot Services table offsets (0x18..0x170, every 8 bytes, 44 entries) --
// Each entry is a function pointer to a trampoline.

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

/// EFI_SUCCESS — the standard "everything is fine" return code.
pub const EFI_SUCCESS: u64 = 0;
/// EFI_NOT_FOUND — returned when a protocol or handle is not found.
pub const EFI_NOT_FOUND: u64 = 0x8000_0000_0000_000E;
/// EFI_BUFFER_TOO_SMALL — the buffer you provided wasn't big enough.
pub const EFI_BUFFER_TOO_SMALL: u64 = 0x8000_0000_0000_0005;

/// Address of the boot-services vtable pointer for dereferencing.
pub const BOOT_SERVICES_VPTR_ADDR: u64 = 0x60;

// ============================================================================
// INSTRUCTION ENCODING CONSTANTS (AArch64)
// ============================================================================

/// NOP (hint) — the universal "do nothing" instruction.
/// Actual encoding: 0xD503_201F.
pub const INSTR_NOP: u32 = 0xD503_201F;

/// RET X30 — return from subroutine.
pub const INSTR_RET: u32 = 0xD65F_03C0;

/// ERET — return from exception.
pub const INSTR_ERET: u32 = 0xD69F_03E0;

/// Base encoding for MOVZ Xd, #imm16 (hw=0).
/// Full encoding: 0xD2_80_0000 | (imm16 << 5) | Rd.
pub const MOVZ_BASE: u32 = 0xD280_0000;

/// Base encoding for MOVK Xd, #imm16 (hw=0).
/// Full encoding: 0xF2_80_0000 | (hw << 21) | (imm16 << 5) | Rd.
pub const MOVK_BASE: u32 = 0xF280_0000;

// -- Exception vector offsets from VBAR_EL1 --

/// Synchronous exception at the current exception level.
pub const VBAR_SYNC_CURRENT_EL: u64 = 0x200;
/// IRQ exception at the current exception level.
pub const VBAR_IRQ_CURRENT_EL: u64 = 0x80;
/// SuperVisor Call (SVC) exception — used by Linux for system calls.
pub const VBAR_SYNC_LOWER_EL_AARCH64: u64 = 0x400;

// ============================================================================
// SYSTEM REGISTER IDS (MRS/MSR encoding)
// ============================================================================
// ARM64 system registers are identified by a 15-bit number composed as:
//   sysreg_id = (op0 << 14) | (op1 << 11) | (CRn << 7) | (CRm << 3) | op2
//
// Example: SCTLR_EL1 = (3,0,1,0,0) → 0b11_000_0001_0000_000 = 0x4108 = 0x4080?

// Actually the encoding is: op0:op1:CRn:CRm:op2 packed into 15 bits.
// ARM reference: op0 is 2 bits (usually 3 for system), op1 3 bits, CRn 4 bits,
// CRm 4 bits, op2 3 bits = 16 bits total (but top bit of op0 is implicit).

// Let me just document them with the actual 16-bit IDs used in MRS/MSR:

/// MIDR_EL1 — Main ID Register (identifies the CPU model).
/// Value returned: 0x410FD083 (Cortex-A72 r0p3).
pub const SYSREG_MIDR_EL1: u16 = 0x4000;

/// MPIDR_EL1 — Multiprocessor Affinity Register.
pub const SYSREG_MPIDR_EL1: u16 = 0x4005;

/// CurrentEL — read the current Exception Level (EL0=0, EL1=1, EL2=2, EL3=3).
pub const SYSREG_CURRENTEL: u16 = 0x4212;

/// ID_AA64PFR0_EL1 — Processor Feature Register 0.
/// Reports AArch64 support at each EL, FP, SIMD, etc.
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
/// Bit 0 enables/disables the MMU.
pub const SYSREG_SCTLR_EL1: u16 = 0x4080;

/// CPACR_EL1 — Architectural Feature Access Control Register.
pub const SYSREG_CPACR_EL1: u16 = 0x4082;

/// TTBR0_EL1 — Translation Table Base Register 0 (user-space page table root).
pub const SYSREG_TTBR0_EL1: u16 = 0x4100;

/// TTBR1_EL1 — Translation Table Base Register 1 (kernel-space page table root).
pub const SYSREG_TTBR1_EL1: u16 = 0x4101;

/// TCR_EL1 — Translation Control Register.
/// Bits [0:5] = T0SZ, bits [16:21] = T1SZ (size of VA space).
pub const SYSREG_TCR_EL1: u16 = 0x4102;

/// SPSR_EL1 — Saved Program Status Register (EL1).
pub const SYSREG_SPSR_EL1: u16 = 0x4200;

/// ELR_EL1 — Exception Link Register (EL1).
/// Holds the return address after an exception.
pub const SYSREG_ELR_EL1: u16 = 0x4201;

/// SP_EL0 — Stack Pointer (EL0).
pub const SYSREG_SP_EL0: u16 = 0x4208;

/// ESR_EL1 — Exception Syndrome Register (EL1).
pub const SYSREG_ESR_EL1: u16 = 0x4290;

/// FAR_EL1 — Fault Address Register (EL1).
pub const SYSREG_FAR_EL1: u16 = 0x4300;

/// MAIR_EL1 — Memory Attribute Indirection Register.
/// Configures how MMU page-table attribute fields map to bus attributes.
pub const SYSREG_MAIR_EL1: u16 = 0x4510;

/// VBAR_EL1 — Vector Base Address Register (EL1).
/// Physical address of the exception vector table.
pub const SYSREG_VBAR_EL1: u16 = 0x4600;

/// TPIDR_EL1 — Thread/Process ID Register (EL1).
pub const SYSREG_TPIDR_EL1: u16 = 0x4684;

/// CTR_EL0 — Cache Type Register.
pub const SYSREG_CTR_EL0: u16 = 0x5801;

/// DCZID_EL0 — Data Cache Zero ID Register (EL0).
pub const SYSREG_DCZID_EL0: u16 = 0x5807;

/// CNTFRQ_EL0 — Counter-Timer Frequency Register.
/// Default: 62.5 MHz (matching QEMU's virt machine).
pub const SYSREG_CNTFRQ_EL0: u16 = 0x5F00;

/// CNTPCT_EL0 — Physical Counter value.
/// Tied to the emulated cycle counter.
pub const SYSREG_CNTPCT_EL0: u16 = 0x5F01;

/// CNTVCT_EL0 — Virtual Counter value.
pub const SYSREG_CNTVCT_EL0: u16 = 0x5F02;

/// TPIDR_EL0 — User Read/Write Thread ID Register.
pub const SYSREG_TPIDR_EL0: u16 = 0x5E82;

/// TPIDRRO_EL0 — User Read-Only Thread ID Register.
pub const SYSREG_TPIDRRO_EL0: u16 = 0x5E83;

/// CNTP_TVAL_EL0 — Physical Timer Value (count-down from this value).
pub const SYSREG_CNTP_TVAL_EL0: u16 = 0x5F10;

/// CNTP_CTL_EL0 — Physical Timer Control.
/// Bit 0 = ENABLE, Bit 1 = IMASK, Bit 2 = ISTATUS.
pub const SYSREG_CNTP_CTL_EL0: u16 = 0x5F11;

/// CNTP_CVAL_EL0 — Physical Timer Compare Value.
pub const SYSREG_CNTP_CVAL_EL0: u16 = 0x5F12;

/// DAIF — Interrupt Mask bits (used by MSR DAIF).
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
/// Returns the ID of the highest-priority pending interrupt, or 1023 if none.
pub const SYSREG_ICC_IAR1_EL1: u16 = 0x4660;

/// ICC_EOIR1_EL1 — End Of Interrupt Register 1.
/// Writing the interrupt ID here tells the GIC the handler is done.
pub const SYSREG_ICC_EOIR1_EL1: u16 = 0x4661;

/// ICC_SRE_EL1 — System Register Enable.
/// Enables system-register access to the GIC CPU interface.
pub const SYSREG_ICC_SRE_EL1: u16 = 0x4665;

// ============================================================================
// FEATURE REGISTER VALUES  (returned for "what CPU is this?" queries)
// ============================================================================

/// MIDR_EL1 value: ARM Cortex-A72 revision r0p3.
pub const MIDR_CORTEX_A72_R0P3: u64 = 0x410FD083;

/// MPIDR_EL1 value: single core, Aff0=0, no multi-threading.
pub const MPIDR_SINGLE_CORE: u64 = 0x80000000;

/// ID_AA64PFR0_EL1: EL0/1/2/3 all support AArch64 (value 1 each).
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

/// Spurious interrupt ID — returned by ICC_IAR1_EL1 when no interrupt is pending.
pub const GIC_SPURIOUS_INTERRUPT: u64 = 1023;

// -- Timer control register bit positions --

/// CNTP_CTL_EL0.ENABLE — set to 1 to start the timer countdown.
pub const TIMER_CTL_ENABLE: u64 = 1;
/// CNTP_CTL_EL0.IMASK — set to 1 to mask (suppress) the timer interrupt.
pub const TIMER_CTL_IMASK: u64 = 1 << 1;
/// CNTP_CTL_EL0.ISTATUS — reads 1 when the timer has expired.
pub const TIMER_CTL_ISTATUS: u64 = 1 << 2;

// ============================================================================
// PROCESSOR STATE (PSTATE) LAYOUT
// ============================================================================

/// Bit position of the N (Negative) flag in PSTATE/SPSR.
pub const PSTATE_N_BIT: u32 = 31;
/// Bit position of the Z (Zero) flag.
pub const PSTATE_Z_BIT: u32 = 30;
/// Bit position of the C (Carry) flag.
pub const PSTATE_C_BIT: u32 = 29;
/// Bit position of the V (Overflow) flag.
pub const PSTATE_V_BIT: u32 = 28;

/// PSTATE mask for the NZCV condition flags (upper nibble of word).
pub const PSTATE_NZCV_MASK: u64 = 0xF000_0000;

/// Bit position of the Exception Level field in PSTATE/SPSR.
pub const PSTATE_EL_SHIFT: u32 = 2;
/// EL mask (2 bits wide).
pub const PSTATE_EL_MASK: u64 = 3 << PSTATE_EL_SHIFT;

/// Bit position of the IRQ mask bit (I) in PSTATE.
pub const PSTATE_I_BIT: u32 = 7;

/// Bits [9:6] in SPSR are the exception-level return mode (RxW, EL).
/// After an exception entry the EL is promoted and these bits capture the old EL.
pub const SPSR_M_MASK: u64 = 0xF << 6;

// ============================================================================
// ARCHITECTURAL CONSTANTS
// ============================================================================

/// Number of general-purpose registers (X0–X30).
pub const NUM_GENERAL_REGISTERS: u8 = 31;

/// The zero register — reading X31 / W31 always returns 0.
pub const ZERO_REGISTER_INDEX: u8 = 31;

/// Stack Pointer register index (encoded as 31 in instructions).
pub const SP_REGISTER_INDEX: u8 = 31;

/// Link register (X30) — holds the return address for BL/BLR.
pub const LINK_REGISTER_INDEX: u8 = 30;

/// Maximum exception level (ARM boots at EL3 with maximum privilege).
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

// ============================================================================
// MMU / PAGE TABLE CONSTANTS
// ============================================================================

/// Virtual address width (48-bit as seen by ARMv8.0-A).
pub const VA_BITS_MAX: u8 = 48;

/// VA split point — kernel VAs start at 0xFFFF_8000_0000_0000.
pub const KERNEL_VA_BASE: u64 = 0xFFFF_8000_0000_0000;

/// Low 32-bit mask used to extract device-identity offset from kernel VA.
pub const VA_LOW32_MASK: u64 = 0xFFFF_FFFF;

/// Number of entries in a page table (512 per level, 9 bits).
pub const PT_ENTRIES: u64 = 512;

/// Shift for page-table index at each level:
/// Level 0 (bits 47:39), Level 1 (38:30), Level 2 (29:21), Level 3 (20:12).
pub const PT_L0_SHIFT: u64 = 39;
pub const PT_L1_SHIFT: u64 = 30;
pub const PT_L2_SHIFT: u64 = 21;
pub const PT_L3_SHIFT: u64 = 12;

/// Size of each level-n block in bytes.
pub const L0_BLOCK_SIZE: u64 = 1 << PT_L0_SHIFT;
pub const L1_BLOCK_SIZE_BYTES: u64 = L1_BLOCK_SIZE; // alias for clarity
pub const L2_BLOCK_SIZE_BYTES: u64 = L2_BLOCK_SIZE;

/// Page table descriptor type bits.
/// Descriptor[1:0] = 0b11 means valid entry.
pub const DESC_VALID: u64 = 0b11;
/// Descriptor[1:0] = 0b01 means block/page (not a table pointer).
pub const DESC_BLOCK: u64 = 0b01;
/// Descriptor[1:0] = 0b11 and bit[1]=1 means table pointer (next level).
pub const DESC_TABLE: u64 = 0b11; // same encoding — distinguished by level
/// Mask to extract the output address from a descriptor.
pub const DESC_ADDR_MASK: u64 = 0x0000_FFFF_FFFF_F000;

/// TCR_EL1.T0SZ / T1SZ field position and mask.
pub const TCR_T0SZ_SHIFT: u64 = 0;
pub const TCR_T0SZ_MASK: u64 = 0x3F;
pub const TCR_T1SZ_SHIFT: u64 = 16;
pub const TCR_T1SZ_MASK: u64 = 0x3F;

/// MAIR_EL1 default value: outer/inner write-back cacheable.
pub const MAIR_EL1_DEFAULT: u64 = 0xFF;

/// SCTLR_EL1 bit 0 — MMU enable.
pub const SCTLR_EL1_M_BIT: u64 = 1;

/// SCTLR_EL1.M — MMU enable bit (same as M_BIT).
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

// ============================================================================
// BOOT PAGE TABLE ADDRESSES
// ============================================================================
// These are pre-allocated page tables set up during EFI boot.
// They are placed in the EFI region above 0x8000_0000.

/// TTBR1 Level-0 table base.
pub const BOOT_TTBR1_L0: u64 = 0x8010_0000;
/// TTBR1 Level-1 table base.
pub const BOOT_TTBR1_L1: u64 = 0x8010_1000;
/// TTBR1 Level-2 table base.
pub const BOOT_TTBR1_L2: u64 = 0x8010_2000;
/// TTBR1 Level-3 table base (first of 96 tables → 96 × 2 MiB = 192 MiB).
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
// EFI STUB TRAP ADDRESSES  (special boot-flow handling)
// ============================================================================
// During the EFI stub phase, some PC addresses get special treatment
// because the EFI services are implemented as traps rather than real calls.

/// PC trap: CopyMem stub (byte-by-byte memory copy).
pub const EFI_TRAP_COPYMEM: u64 = 0x8000_CE00;
/// PC trap: SetMem stub (byte-by-byte memory fill).
pub const EFI_TRAP_SETMEM: u64 = 0x8000_D000;
/// PC trap: AllocatePages stub (bump allocator return).
pub const EFI_TRAP_ALLOCPAGES: u64 = 0x8000_D200;
/// PC trap: FreePages stub (no-op return).
pub const EFI_TRAP_FREEPAGES: u64 = 0x8000_D400;

// -- Cache invalidation loop fast-forwards --
// These are busy-loops that clear vast ranges of cache; we skip them.

/// Cache maintenance loop entry PC.
pub const CACHE_INV_LOOP_ENTRY: u64 = 0x400b6e80;
/// Cache maintenance loop exit PC (where we jump to after fast-forward).
pub const CACHE_INV_LOOP_EXIT: u64 = 0x400b6e90;
/// Instruction cache invalidate loop entry PC.
pub const I_CACHE_INV_LOOP_ENTRY: u64 = 0x400b6eb8;
/// Instruction cache invalidate loop exit PC.
pub const I_CACHE_INV_LOOP_EXIT: u64 = 0x400b6ec8;

// ============================================================================
// PE/COFF HEADER CONSTANTS
// ============================================================================

/// ARM64 Linux kernel magic: `"ARM\x64"` in little-endian ASCII → 0x644D5241.
pub const ARM64_KERNEL_MAGIC: u32 = 0x644d5241;

/// PE32+ optional header magic (0x20B).
pub const PE32PLUS_MAGIC: u16 = 0x020B;

/// `PE\0\0` signature at the start of a PE/COFF image.
pub const PE_SIGNATURE: &[u8; 4] = b"PE\0\0";

/// Offset in the kernel Image where the PE signature is expected.
pub const KERNEL_PE_OFFSET: usize = 0x40;

/// Minimum size of a PE optional header for our purposes.
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

/// Begin Node token.
pub const FDT_BEGIN_NODE: u32 = 0x0000_0001;
/// End Node token.
pub const FDT_END_NODE: u32 = 0x0000_0002;
/// Property token.
pub const FDT_PROP: u32 = 0x0000_0003;
/// End-of-structure token.
pub const FDT_END: u32 = 0x0000_0009;

// ============================================================================
// CPIO (INITRD) CONSTANTS
// ============================================================================

/// cpio newc format magic: "070701" or "070702" (CRC).
pub const CPIO_NEWC_MAGIC: &str = "070701";

/// cpio newc trailer file name — marks the end of the archive.
pub const CPIO_TRAILER_NAME: &str = "TRAILER!!!";

/// Size of a cpio newc header in bytes (110).
pub const CPIO_HEADER_SIZE: usize = 110;

// ============================================================================
// MISC
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

/// Maximum history entries kept for instruction tracing.
pub const INSTR_HISTORY_SIZE: usize = 100;

/// Timeslice: number of instructions each core runs before yielding.
pub const ROUND_ROBIN_TIMESLICE: usize = 10_000;
