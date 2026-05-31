# WebBoxVM — Vision

A high-fidelity ARM64 virtual machine running in the browser via WebAssembly. The project aims to boot real operating systems — starting with Linux, eventually Windows — entirely client-side.

## Why

Running an OS in the browser means instant access from any device, zero server cost, and complete privacy (everything stays local). ARM64 is the obvious target: it's the architecture of every modern phone, Apple Silicon Macs, and the growing Snapdragon laptop ecosystem.

## Current State

The emulator boots ARM64 Linux through the standard ARM64 Image protocol:
X0 points at the device tree, X1-X3 are zero, the CPU enters at EL1 with the MMU off, and Linux enables its own virtual address space.

The native CLI path now boots a standard Debian ARM64 netinst ISO far enough to start the real serial text installer. The validated run reaches `/lib/debian-installer/menu`, executes `/usr/bin/main-menu`, and prints the installer language prompt:

```text
Choose the language to be used for the installation process.
Language:
```

The browser path now has a concrete application shell: a WASM build, xterm.js serial console, ISO picker, Debian boot target, UART keyboard input, pause/resume/reset controls, and live VM metrics. Sparse guest memory keeps browser builds from reserving the full guest memory layout up front.

The next milestone is proving the same Debian installer prompt inside the browser app with responsive input, then tightening performance enough that the experience feels like an interactive terminal rather than a long-running trace.

## Principles

1. **Linux first, then Windows.** Prove the emulator on a smaller, well-understood kernel before tackling a complex OS.
2. **Text before graphics.** A reliable UART console before WebGPU framebuffer.
3. **Interpreter before JIT.** Correctness first, performance second.
4. **Test everything.** One test per instruction, one test per device register.
5. **Clean architecture.** Modular Rust with no global state, clear ownership, and self-documenting constants.

## Targets

1. **Linux early UART** — boot Linux to serial output and prove the kernel can talk back
2. **Standard ISO terminal** — boot a normal ARM64 Linux ISO to a real terminal installer environment
3. **Browser terminal** — run the ISO path through WebAssembly with xterm.js input/output
4. **Linux shell/install workflow** — interact with BusyBox or Debian installer screens reliably
5. **Windows PE loader** — parse Windows boot structures, load `ntoskrnl.exe`
6. **Windows desktop** — boot Windows 11 ARM64 to a usable desktop in the browser

## License

AGPL-3.0 for open source. Commercial licensing available for proprietary use.
