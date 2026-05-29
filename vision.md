# WebBoxVM — Vision

A high-fidelity ARM64 virtual machine running in the browser via WebAssembly. The project aims to boot real operating systems — starting with Linux, eventually Windows — entirely client-side.

## Why

Running an OS in the browser means instant access from any device, zero server cost, and complete privacy (everything stays local). ARM64 is the obvious target: it's the architecture of every modern phone, Apple Silicon Macs, and the growing Snapdragon laptop ecosystem.

## Current State

The emulator boots an ARM64 Linux kernel (6.6.70, custom-built) through the full boot chain:
PE/COFF entry → UEFI firmware → EFI stub → kernel handoff → MMU enable → kernel virtual address space.

The kernel reaches its virtual address space, configures exception vectors, and receives timer interrupts. The next milestone is reaching `start_kernel()` for UART console output and an interactive shell.

## Principles

1. **Linux first, then Windows.** Prove the emulator on a smaller, well-understood kernel before tackling a complex OS.
2. **Text before graphics.** UART console before WebGPU framebuffer.
3. **Interpreter before JIT.** Correctness first, performance second.
4. **Test everything.** One test per instruction, one test per device register.
5. **Clean architecture.** Modular Rust with no global state, clear ownership, and self-documenting constants.

## Targets

1. **Linux shell** — boot Debian ARM64 to `ash`, run commands, prove the emulator works
2. **Windows PE loader** — parse Windows boot structures, load `ntoskrnl.exe`
3. **Windows desktop** — boot Windows 11 ARM64 to a usable desktop in the browser

## License

AGPL-3.0 for open source. Commercial licensing available for proprietary use.
