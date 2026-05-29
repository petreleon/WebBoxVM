# WebBoxVM — Vision

A client-side **Windows 11 ARM64** virtual machine running in the browser via WebAssembly and WebGPU.

## Why

Cloud VMs are expensive and slow to boot. Existing browser emulators (v86) lack ARM64 and hardware graphics. WebBoxVM runs entirely client-side with zero hosting cost — and targets the fastest-growing laptop architecture (Snapdragon X Elite, Apple Silicon via virtualization).

Running **Windows 11 ARM64** in the browser means:
- Native ARM64 app compatibility (no x86 emulation overhead)
- Zero-latency client-side compute
- A full desktop OS accessible from any browser tab

## Core Principles

1. **Windows 11 ARM64 is the main objective.** Linux is the stepping stone, not the destination.
2. **No graphics until text works.** UART console before WebGPU framebuffer.
3. **No JIT until interpreter works.** Measure first, optimize second.
4. **One test per instruction.** No untrusted code paths.

## Target

**Primary:** Boot a retail Windows 11 ARM64 ISO to the desktop in the browser.

**Milestone 0 (COMPLETE):** ARM64 CPU interpreter — 90 opcodes decoded, MMU with 3-level page table walk + 2048-entry TLB, PL011 UART with 7 kernel-path tests, GICv3 interrupt controller, EFI firmware with real AllocatePages/CopyMem/SetMem trampolines.

**Milestone 1 (IN PROGRESS):** Boot an ARM64 Linux kernel to an interactive shell.
- ✅ Boot chain complete: PE entry → EFI stub (3.5M steps) → handoff → primary_entry → MMU enable → kernel VA space
- ✅ VBAR_EL1 configured by kernel, timer IRQ fires, handler runs, ERET returns
- ⬜ Kernel stuck in pre-`start_kernel` init spin loop — investigating hardware condition
- ⬜ UART output via `earlycon=pl011,0x09000000 console=ttyAMA0`
- ⬜ Interactive shell: `ls`, `echo hello`, `cat /proc/cpuinfo`

**Milestone 2:** Boot Windows 11 ARM64 PE loader and kernel initialization.

**Milestone 3:** Reach Windows desktop with basic input (keyboard/mouse) and display.

## Architecture

- **CPU**: Pre-decoded threaded interpreter for AArch64. Must eventually cover all ARMv8-A + ARMv8.2-A instructions including NEON, Crypto, and ARMv8.1 Atomics.
- **Memory**: 1 GiB contiguous `Vec<u8>` (physical only until MMU needed). Will need 4–8 GiB for Windows.
- **Devices**: PL011 UART at `0x0900_0000`. GICv3 (required for Windows). ACPI tables (required for Windows). TPM 2.0 (required for Windows 11). VirtIO GPU later.
- **Storage**: OPFS for disk images. Windows requires NVMe or SATA controller emulation.
- **Display**: Xterm.js for serial, HTML5 Canvas/WebGPU for framebuffer.

## License

AGPL-3.0. Commercial licensing available on request.
