# WebBoxVM — Todo

## Sprint 1 — CPU Core (This Week)
- [x] Initialize Rust workspace with `emulator` crate
- [x] Define `RegisterFile`, `PState`, `Armv8Cpu`
- [ ] Implement instruction decoder (ADD, SUB, MOVZ, LDP, LDR, STR)
- [ ] Implement execute loop (`cpu.step(memory) -> cycles`)
- [ ] Physical memory: 1 GiB flat array + MMIO dispatch table
- [ ] Unit test: ADD X0, X1, X2
- [ ] Unit test: MOVZ X5, #0x1234, LSL #16
- [ ] Unit test: LDR/STR roundtrip

## Sprint 2 — Bootloader (Next Week)
- [ ] ELF loader: parse vmlinuz Image header
- [ ] Load kernel into RAM at 0x4008_0000
- [ ] Minimal boot stub at 0x0: set EL1, jump to kernel
- [ ] UART console: write character to stdout
- [ ] See "Uncompressing Linux..." on terminal

## Sprint 3 — Devices
- [ ] GICv2 distributor stub (enough for timer)
- [ ] ARM generic timer (CNTPCT increments)
- [ ] Device tree blob (DTB) in memory

## Sprint 4 — MMU
- [ ] 3-level page table walk
- [ ] 2048-entry TLB
- [ ] SCTLR_EL1 enables MMU

## Sprint 5 — Shell
- [ ] Busybox initrd boots
- [ ] Interactive shell prompt
- [ ] `ls`, `echo hello`, `cat /proc/cpuinfo`

## Backlog (Do Not Touch)
- x86_64 interpreter
- WebGPU / VirtIO GPU
- JIT compiler
- OPFS disk
- Networking
- Multicore
- Commercial licensing
