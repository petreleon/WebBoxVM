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

## Sprint 5 — MMU
- [ ] 3-level page table walk (39-bit VA)
- [ ] 2048-entry software TLB
- [ ] `SCTLR_EL1` enables MMU

## Sprint 6 — Busybox Shell
- [ ] Initrd: load cpio ramdisk into memory
- [ ] Kernel boots to Busybox `ash` shell
- [ ] Interactive: `ls`, `echo hello`, `cat /proc/cpuinfo`

**Result:** 69 tests pass, 0 compiler warnings.

## Backlog — Do Not Touch Until Shell Works
- x86_64 interpreter (QEMU or from scratch)
- VirtIO GPU / WebGPU display
- JIT compilation (x86→Wasm)
- OPFS persistent disk
- Networking (VirtIO Net + wsproxy)
- Multicore (SMP)
- Commercial licensing / dual-license
