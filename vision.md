# WebBoxVM — Vision

A high-fidelity ARM64 virtual machine running in the browser via WebAssembly. The project aims to boot real operating systems — starting with Linux, eventually Windows — entirely client-side.

## Why

Running an OS in the browser means instant access from any device, zero server cost, and complete privacy (everything stays local). ARM64 is the obvious target: it's the architecture of every modern phone, Apple Silicon Macs, and the growing Snapdragon laptop ecosystem.

## Current State

The emulator boots an ARM64 Linux kernel (6.6.70, custom-built) through the standard ARM64 Image protocol:
X0 points at the device tree, X1-X3 are zero, the CPU enters at EL1 with the MMU off, and Linux enables its own virtual address space.

The kernel now reaches early PL011 UART output and prints the first Linux boot log lines, including `Linux version 6.6.70`, `Machine model: WebBoxVM`, and `earlycon: pl11`.

The next milestone is continuing from early console to initramfs unpacking, `/init`, and an interactive BusyBox shell.

## Principles

1. **Linux first, then Windows.** Prove the emulator on a smaller, well-understood kernel before tackling a complex OS.
2. **Text before graphics.** A reliable UART console before WebGPU framebuffer.
3. **Interpreter before JIT.** Correctness first, performance second.
4. **Test everything.** One test per instruction, one test per device register.
5. **Clean architecture.** Modular Rust with no global state, clear ownership, and self-documenting constants.

## Targets

1. **Linux early UART** — boot Linux to serial output and prove the kernel can talk back
2. **Linux shell** — boot ARM64 Linux to BusyBox `ash`, run commands, prove the emulator is useful
3. **Windows PE loader** — parse Windows boot structures, load `ntoskrnl.exe`
4. **Windows desktop** — boot Windows 11 ARM64 to a usable desktop in the browser

## License

AGPL-3.0 for open source. Commercial licensing available for proprietary use.
