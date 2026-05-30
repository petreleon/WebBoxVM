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

**Result at completion:** 23 tests passed.

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

**Result at completion:** 40 tests passed (1 slow test ignored).

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

## Sprint 6 — Linux Early UART Boot (COMPLETE)
- [x] Initrd: load cpio ramdisk into memory
- [x] Exclusive load/store (LDXR/LDXP/STXR/STXP/LDAR/STLR) decode & execute
- [x] DTB: GICv3 interrupt controller node, ARMv8 timer node, UART interrupts, `interrupt-parent`
- [x] Bootargs: `earlycon=pl011,0x09000000 console=ttyAMA0 rdinit=/init`
- [x] PL011 UART: full register emulation (DR, FR, CR, IBRD, FBRD, LCR_H, IFLS, IMSC, RIS, MIS, ICR, DMACR) with 7 unit tests
- [x] EFI services: AllocatePages (real ARM64 bump-allocator trampoline), CopyMem, SetMem, HandleProtocol, GetMemoryMap
- [x] PE header parsing: dynamically read PE entry_RVA from optional header
- [x] Custom kernel built via Docker (6.6.70, `CONFIG_RELOCATABLE=y`)
- [x] Standard ARM64 Linux Image boot protocol: X0=DTB, X1-X3=0, EL1, MMU off, IRQs masked
- [x] Boot chain complete: Image entry → `primary_entry` → MMU enable → kernel VA space
- [x] WFI/WFE decode & execute (fast-forward cycle counter)
- [x] DAIFSet/DAIFClr decode & execute (IRQ mask control)
- [x] Conditional compare correctness: CCMP/CCMN register and immediate forms
- [x] Signed load correctness: LDRSB/LDRSH/LDRSW sign extension
- [x] ARMv8.1 LSE atomics: LDADD/LDSET/CAS/CASP decode and execute paths
- [x] Pair exclusive decode: LDXP/STXP/STLXP paths
- [x] Generic timer control: CNTP_CTL_EL0 enable/mask/status semantics
- [x] Timer IRQ delivery honors PSTATE.I and uses current-EL SPx vector offset
- [x] VBAR_EL1 confirmed set by kernel, IRQ handler runs when unmasked
- [x] EL1 kernel entry state confirmed during init
- [x] Kernel prints early UART boot log:
  - `[    0.000000] Booting Linux on physical CPU 0x0000000000 [0x410fd083]`
  - `[    0.000000] Linux version 6.6.70 ...`
  - `[    0.000000] Machine model: WebBoxVM`
  - `[    0.000000] earlycon: pl11 at MMIO 0x0000000009000000`

**Result:** Linux 6.6.70 reaches early PL011 UART output through the standard ARM64 Image boot path. `cargo run --example wait_uart --release` prints kernel boot messages by 4M emulated steps with zero fetch/execute faults.

## Sprint 7 — Busybox Shell (IN PROGRESS)
- [ ] Kernel boots to Busybox `ash` shell
  - Early console works; next target is enough init, scheduler, device, and initrd behavior to spawn `/init`
  - Current initrd contains placeholder BusyBox bytes and a minimal `/init` script
- [ ] Replace placeholder BusyBox payload with a real static ARM64 BusyBox binary
- [ ] Continue boot beyond early console into initramfs unpacking and `/init`
- [ ] Add UART RX path wiring for interactive shell input
- [ ] **Standard boot for CONFIG_RELOCATABLE=n kernels** — real bootloaders (U-Boot/GRUB) don't relocate:
  - [ ] Add kernel `PAGE_OFFSET` (e.g. `0xffff800000000000`) to TTBR1 identity mapping
  - [ ] Map kernel VA range → physical load address BEFORE EFI stub runs
  - [ ] When EFI stub checks `_text == *image_addr`, both match → `EFI_SUCCESS`
  - [ ] Kernel boots at its linked VA with MMU already active (no relocation needed)
  - [ ] Works for all pre-built Debian/Ubuntu kernels without Docker rebuild
- [ ] Interactive: `ls`, `echo hello`, `cat /proc/cpuinfo`

---

## Future Work

These are aspirational targets, not committed sprints. Most depend on the Linux shell milestone completing first.

### ISA Completeness
- [ ] NEON / SIMD (load/store, vector arithmetic)
- [ ] Crypto extensions (AES, SHA)
- [ ] Harden ARMv8.1 LSE atomics coverage, including memory ordering edge cases
- [ ] Remaining system instructions

### Devices & Firmware
- [ ] ACPI tables (RSDP, DSDT, MADT, GTDT, FADT)
- [ ] NVMe or VirtIO storage controller
- [ ] TPM 2.0 MMIO stub
- [ ] Framebuffer (VirtIO GPU or simple linear FB)
- [ ] Keyboard/mouse input via HID

### Windows 11 ARM64
- [ ] Parse Windows ISO / WIM
- [ ] Boot Windows PE → kernel loader → `ntoskrnl.exe`
- [ ] Reach desktop with display + input
- [ ] Network adapter (VirtIO Net)

### WebAssembly Target
- [ ] Compile to wasm64 + wasm-bindgen
- [ ] Browser deployment with xterm.js console
- [ ] OPFS persistent disk for browser storage

## Backlog — General
- Multi-core (SMP)
- JIT compilation (ARM64 → native)
- x86_64 interpreter (for broader OS compatibility)
- Commercial licensing / dual-license
