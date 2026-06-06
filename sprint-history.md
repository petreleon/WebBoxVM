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

## Sprint 7 — Serial Linux Userspace and ISO Installer
- Replaced the placeholder initrd payload with static ARM64 BusyBox.
- Booted into initramfs userspace and spawned `/init`.
- Wired PL011 UART RX so the serial console accepts interactive input.
- Added ISO9660 extraction for ARM64 Linux installer media.
- Attached ISO media as a read-only VirtIO block device.
- Added a second writable sparse VirtIO block device for installer storage.
- Added sparse install-disk snapshot/restore support.
- Validated Debian ARM64 netinst through `/lib/debian-installer/menu` and `/usr/bin/main-menu`.

**Result:** Native CLI boots Linux to BusyBox `ash`; Debian ARM64 netinst reaches the real serial text installer language prompt.

## Sprint 8 — Browser Terminal Delivery
- Added the `web/` application shell with xterm.js terminal output.
- Added ISO picker, Debian boot target, pause/resume/reset, step-slice, and live metrics.
- Routed browser keyboard input to PL011 UART RX.
- Added browser install-disk size controls and sparse allocation metrics.
- Persisted install disk snapshots through OPFS.
- Verified browser interaction at the Debian installer prompt.

**Result:** The browser app can boot Debian ARM64 netinst to the serial installer prompt with persistent sparse disk support.

## Sprint 9 — Wasm64 Browser Target
- Switched browser builds to `wasm64-unknown-unknown`.
- Built the package with nightly `build-std` and `wasm-bindgen`.
- Added Memory64 capability detection before VM boot.
- Verified wasm64 package instantiation and byte-array boot input.

**Result:** WebBoxVM is wasm64-first for browser builds and requires WebAssembly Memory64 support.

## Sprint 10 — Browser Worker Execution
- Moved the wasm64 `Emulator` instance into a module Web Worker.
- Added a main-thread VM proxy with cached metrics.
- Moved the run pump into the worker.
- Routed UART, live metrics, errors, and disk snapshot operations across the worker boundary.

**Result:** Browser execution no longer runs guest steps on the UI thread.

## Sprint 11 — Debian Installer Component Loading
- Diagnosed a browser Wasm64 JIT slowdown/stall from EL0 guest-memory helper blocks.
- Restored conservative skipping for EL0 helper blocks while keeping pending-store forwarding for helper correctness.
- Used `disarm64`-backed decoding to classify `0x6e20ac00` as `uminp`, already decoded as `Opcode::SimdUminp`.
- Split SIMD opcode display names into `emulator/src/arm64/opcodes/names_simd.rs`.
- Verified Debian ARM64 netinst loads installer components from ISO media to 100% in the browser.
- Verified the installer advances to `Detecting network hardware`.

**Result:** The previous "Load installer components from installation media" failure is fixed; the next concrete area is network/device behavior and performance.
