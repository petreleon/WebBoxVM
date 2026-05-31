# WebBoxVM — Sprint History

## Sprint 1 — CPU Core
- Initialized the Rust workspace with the `emulator` crate.
- Defined register, processor state, and `Armv8Cpu` foundations.
- Implemented the first instruction decoder and execute loop.
- Added RAM, MMIO dispatch, and PL011 UART serial output.
- Covered ADD, SUB, MOVZ, LDR/STR, branch, and hello-UART tests.

**Result:** 23 tests passed.

## Sprint 2 — Bootloader
- Added boot stub mechanics so `BR Xn` can jump to a kernel entry point.
- Downloaded and inspected a real Debian ARM64 PE/EFI kernel.
- Implemented branch, compare, pair load/store, ADRP, conditional branch, literal load, and barrier instructions needed by the PE stub.
- Added SP support in register access and memory addressing.
- Parsed the ARM64 Linux kernel header and PE section table.
- Loaded the kernel into RAM and built the first real-kernel execution trace.
- Added a synthetic kernel test that prints `Uncompressing Linux...`.

**Result:** 40 tests passed, with one slow test ignored.

## Sprint 3 — EFI Stub Protocols
- Split EFI code into focused encode, layout, table, and module files.
- Implemented minimal EFI System Table, Boot Services, Runtime Services, and success-return trampolines.
- Extended physical memory with a dedicated EFI region.
- Implemented Loaded Image Protocol, `AllocatePages`, `FreePages`, `GetMemoryMap`, and `ExitBootServices` callbacks.
- Traced protocol offsets used by the Debian stub and verified return values.
- Booted the real kernel past EFI stub setup to the decompressor handoff.

**Result:** Real kernel executes 200+ PE-stub instructions without crashing.

## Sprint 4 — PE Relocations and Decompressor
- Split large instruction and interpreter files into smaller decode, execute, opcode, helper, and test modules.
- Deduplicated memory read/write selection helpers.
- Implemented PE32+ `.reloc` parsing and DIR64 fixups.
- Split loader code into kernel, relocation, and module files.
- Added relocation tests for parse, DIR64, no-delta, and zero-size cases.
- Identified decompressor entry and booted past EFI stub into the kernel virtual entry path.
- Added missing decompressor-sensitive ALU, branch, system register, multiply, and sign-extension behavior.

## Sprint 5 — MMU
- Implemented 3-level page table walk for 39-bit virtual addresses.
- Added a 2048-entry software TLB.
- Honored `SCTLR_EL1.M` for MMU enable/disable.

**Result:** 80 tests passed with zero compiler warnings.

## Sprint 6 — Linux Early UART Boot
- Loaded cpio initrd data into guest memory.
- Implemented exclusive load/store, conditional compare, signed load, and ARMv8.1 LSE atomic paths needed by Linux.
- Generated DTB nodes for RAM, CPUs, timer, GIC, UART, chosen bootargs, and initrd metadata.
- Completed PL011 UART register emulation and RX/TX behavior.
- Added EFI `AllocatePages`, `CopyMem`, `SetMem`, `HandleProtocol`, and `GetMemoryMap` support.
- Parsed PE entry RVA dynamically from the optional header.
- Built and booted a custom relocatable Linux 6.6.70 ARM64 Image.
- Followed the standard ARM64 Image protocol into `primary_entry`, MMU enable, kernel VA space, and early console output.
- Implemented WFI/WFE, DAIFSet/DAIFClr, timer IRQ status, masking, and current-EL vector delivery behavior.

**Result:** Linux reaches early PL011 UART output through the standard ARM64 Image boot path.
