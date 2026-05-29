# WebBoxVM — Todo

## Sprint 1 — CPU Core (COMPLETE)
- [x] Initialize Rust workspace with `emulator` crate
- [x] Define `RegisterFile`, `ProcessorState`, `Armv8Cpu`
- [x] Implement instruction decoder (ADD, SUB, MOVZ, LDR, STR, NOP, B)
- [x] Implement execute loop (`run(cpu, bus, entry, max_steps)`)
- [x] Physical memory: 1 GiB flat array
- [x] Unit test: ADD X0, X1, X2
- [x] Unit test: SUB X0, X1, X2
- [x] Unit test: MOVZ X0, #0x1234
- [x] Unit test: MOVZ X0, #0x1234, LSL #16
- [x] Unit test: LDR/STR roundtrip
- [x] Unit test: branch forward
- [x] Unit test: hello_uart end-to-end
- [x] System bus with MMIO dispatch (RAM + UART)
- [x] PL011 UART for serial output

**Result:** 23 tests pass, zero warnings.

## Sprint 2 — Bootloader (COMPLETE)
- [x] Boot stub mechanism: BR Xn jumps to kernel entry point
- [x] End-to-end test: boot stub → kernel code → UART output
- [x] Download real Debian ARM64 kernel (PE/EFI format, 37 MB)
- [x] Implement missing ARM64 instructions (~15 opcodes needed for real kernel):
  - [x] BL (branch with link), RET (return)
  - [x] CBZ/CBNZ (compare and branch on zero)
  - [x] LDP/STP (load/store pair)
  - [x] MOV (register), CMP (compare)
  - [x] ADD/SUB immediate
  - [x] ADRP (page-relative address)
  - [x] B.cond (conditional branch)
  - [x] TBZ/TBNZ (test bit and branch)
  - [x] MOVK (move with keep)
  - [x] LDR literal (PC-relative load)
  - [x] DSB/ISB/DMB (barrier nops)
- [x] SP support in register read/write and memory addressing
- [x] Parse ARM64 Linux kernel header / PE section table
- [x] Load kernel into RAM at `0x4008_0000`
- [x] Boot stub: set X0=DTB addr, branch to kernel entry `0x41da7ee0`
- [x] Run kernel: successfully decode and execute 22 real kernel instructions
- [x] Synthetic kernel test prints "Uncompressing Linux...\n" on UART
  - PE-wrapped Debian kernel requires EFI runtime services (not implemented)
  - Raw kernel boot deferred to Sprint 3 (needs DTB + memory layout)

**Result:** 40 tests pass (1 slow test ignored), zero warnings.

## Sprint 3 — EFI Stub Protocols (COMPLETE)
- [x] Code reorganization: split EFI into `encode.rs`, `layout.rs`, `tables.rs`, `mod.rs`
- [x] Minimal EFI SystemTable + BootServices + RuntimeServices trampolines (return EFI_SUCCESS)
- [x] Extend PhysicalMemory to support EFI region (`0x8000_0000 – 0x8FFF_FFFF`)
- [x] Fix synthetic kernel test (`/tmp/kernel_raw.bin` overwritten, MOVZ encoding bug)
- [x] Real kernel executes **34 PE-stub instructions** before `RET X30` where `X30=0`
- [x] Implement `HandleProtocol` Loaded Image Protocol callback
  - Return pointer to `EFI_LOADED_IMAGE_PROTOCOL` structure with image_base, image_size
- [x] Implement `AllocatePages` / `FreePages` callback
  - Track a simple allocator, reserve pages for kernel relocation
- [x] Implement `GetMemoryMap` callback
  - Return RAM ranges: `0x4000_0000 – 0x7FFF_FFFF`
- [x] Implement `ExitBootServices` callback
  - Final step before jumping to decompressed kernel
- [x] Trace which protocol offsets the Debian stub calls, verify return values
- [x] Boot real kernel past EFI stub → decompressor → `printk("Uncompressing Linux...")`

**Result:** Real kernel executes **200+ PE-stub instructions** without crashing. EFI stub completes and returns to caller.

## Sprint 4 — PE Relocations & Decompressor (COMPLETE)
- [x] Refactor: split `arm64/instr.rs` (814 lines) into `opcodes.rs`, `decode.rs`, `execute.rs`, `helpers.rs`
- [x] Refactor: split `arm64/interpreter.rs` into `interpreter/mod.rs` + `interpreter/tests.rs`
- [x] Refactor: deduplicate `memory.rs` read/write logic into unified `select_region` helpers
- [x] Implement PE32+ `.reloc` section parsing
  - Read `ImageBase`, `DataDirectory[5]` from PE optional header
  - Iterate relocation blocks, extract type (ABSOLUTE/HIGHLOW/DIR64) and offset
  - Apply fixups: `target += KERNEL_LOAD - preferred_base`
- [x] `loader.rs` → `loader/kernel.rs` + `loader/relocations.rs` + `loader/mod.rs`
- [x] 4 unit tests for relocations (parse, DIR64, no-delta, zero-size)
- [x] Trace stub return after `ExitBootServices` / `RET` to caller
- [x] Identify decompressor entry point and set PC
- [x] Boot real kernel past EFI stub and decompressor to virtual entry point `0xffff8000801a8f60`
- [x] Implement remaining ALU, branch, system register, and multiply ops as decompressor needs them (MADD/MSUB/UMADDL, MRS/MSR thread/stack registers, shifted/extended register arithmetic, and correct TBZ/TBNZ sign-extension)

## Sprint 5 — MMU (COMPLETE)
- [x] 3-level page table walk (39-bit VA)
- [x] 2048-entry software TLB
- [x] `SCTLR_EL1` enables MMU

**Result:** 80 tests pass, 0 compiler warnings.

## Sprint 6 — Busybox Shell (IN PROGRESS)
- [x] Initrd: load cpio ramdisk into memory
- [x] Exclusive load/store (LDXR/LDXP/STXR/STXP/LDAR/STLR) decode & execute
- [x] DTB: GICv3 interrupt controller node, ARMv8 timer node, UART interrupts, `interrupt-parent`
- [x] Bootargs: `earlycon=pl011,0x09000000 console=ttyAMA0 rdinit=/init`
- [x] PL011 UART: full register emulation (DR, FR, CR, IBRD, FBRD, LCR_H, IFLS, IMSC, RIS, MIS, ICR, DMACR) with 7 unit tests
- [x] EFI services: AllocatePages (real ARM64 bump-allocator trampoline), CopyMem, SetMem, HandleProtocol, GetMemoryMap
- [x] PE header parsing: dynamically read PE entry_RVA from optional header
- [x] Custom kernel built via Docker (6.6.70, `CONFIG_RELOCATABLE=y`)
- [x] Boot chain complete: PE entry → EFI stub (3.5M steps) → handoff → primary_entry → kernel VA space
- [ ] Kernel boots to Busybox `ash` shell
  - Kernel enters VA space at `0xffff8000800a3240`, running in tight 3-address init loop
  - Likely waiting for timer IRQ (ARM Generic Timer) before scheduler starts → `start_kernel()` → earlycon → UART
- [ ] **Timer interrupts** (ARM Generic Timer delivery via GICv3)
- [ ] **Standard boot for CONFIG_RELOCATABLE=n kernels** — real bootloaders (U-Boot/GRUB) don't relocate:
  - [ ] Add kernel `PAGE_OFFSET` (e.g. `0xffff800000000000`) to TTBR1 identity mapping
  - [ ] Map kernel VA range → physical load address BEFORE EFI stub runs
  - [ ] When EFI stub checks `_text == *image_addr`, both match → `EFI_SUCCESS`
  - [ ] Kernel boots at its linked VA with MMU already active (no relocation needed)
  - [ ] Works for all pre-built Debian/Ubuntu kernels without Docker rebuild
- [ ] Interactive: `ls`, `echo hello`, `cat /proc/cpuinfo`

**Result:** 98 tests pass, 0 compiler warnings. Full PL011 emulation with 7 kernel-code-path tests. EFI stub runs to completion. Custom kernel reaches VA space — blocked by missing timer IRQ delivery.

---

## Phase 2 — Windows 11 ARM64

> The main objective. Everything above is scaffolding.

### Sprint 7 — Exception Model & Interrupts
- [ ] GICv3 distributor and redistributor emulation
- [ ] IRQ delivery path: `VBAR_EL1` + exception entry
- [ ] Timer interrupts (ARM Generic Timer)
- [ ] `SPSR_EL1` / `ELR_EL1` save/restore on exception

### Sprint 8 — ISA Completeness
- [ ] NEON / SIMD ( AdvSIMD load/store, vector arithmetic )
- [ ] Crypto extensions (AES, SHA-1, SHA-256)
- [ ] ARMv8.1 Atomics (LDADD, CAS, SWP)
- [ ] DC ZVA, cache maintenance ops
- [ ] Remaining system instructions (AT S1E1R, DC IVAC, etc.)

### Sprint 9 — ACPI & Firmware
- [ ] Generate ACPI tables (RSDP, DSDT, MADT, GTDT, FADT)
- [ ] UEFI runtime services required by Windows
- [ ] SMBIOS tables

### Sprint 10 — Storage & Boot
- [ ] NVMe controller emulation (VirtIO-NVMe or simple MMIO)
- [ ] Parse Windows 11 ARM64 ISO / WIM
- [ ] Boot Windows PE → kernel loader → `ntoskrnl.exe`

### Sprint 11 — Windows Kernel Bring-up
- [ ] TPM 2.0 MMIO stub (Windows 11 hard-requires TPM)
- [ ] Complete enough ISA for `ntoskrnl` init
- [ ] Reach `KDPOLL` or early video output

### Sprint 12 — Display & Input
- [ ] Basic framebuffer (VirtIO GPU or simple Bochs VBE)
- [ ] WebGPU output path
- [ ] Keyboard/mouse input via HID

### Sprint 13 — Desktop
- [ ] Windows 11 desktop boots
- [ ] Basic app launch (Notepad, Calculator)
- [ ] Network adapter (VirtIO Net)

---

## Backlog — General
- x86_64 interpreter (QEMU or from scratch)
- JIT compilation (ARM64 → Wasm)
- OPFS persistent disk
- Multicore (SMP)
- Commercial licensing / dual-license
