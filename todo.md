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
- [ ] Run kernel, see first output on UART (deferred: needs more instructions + memory init)

**Result:** 40 tests pass (1 slow test ignored), zero warnings.

## Sprint 3 — Device Tree & Memory Layout
- [ ] GICv2 distributor stub (enough for timer IRQ 30)
- [ ] ARM Generic Timer (`CNTPCT` increments, comparator fires)
- [ ] Device tree blob (DTB) in guest memory

## Sprint 4 — MMU
- [ ] 3-level page table walk (39-bit VA)
- [ ] 2048-entry software TLB
- [ ] `SCTLR_EL1` enables MMU

## Sprint 5 — Busybox Shell
- [ ] Initrd: load cpio ramdisk into memory
- [ ] Kernel boots to Busybox `ash` shell
- [ ] Interactive: `ls`, `echo hello`, `cat /proc/cpuinfo`

## Backlog — Do Not Touch Until Shell Works
- x86_64 interpreter (QEMU or from scratch)
- VirtIO GPU / WebGPU display
- JIT compilation (x86→Wasm)
- OPFS persistent disk
- Networking (VirtIO Net + wsproxy)
- Multicore (SMP)
- Commercial licensing / dual-license
