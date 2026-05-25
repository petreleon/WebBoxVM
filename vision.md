# WebBoxVM — Vision

A client-side ARM64 virtual machine running in the browser via WebAssembly and WebGPU.

## Why

Cloud VMs are expensive and slow to boot. Existing browser emulators (v86) lack ARM64 and hardware graphics. WebBoxVM runs entirely client-side with zero hosting cost.

## Core Principles

1. **Ship ARM64 first.** No x86_64 until Linux boots on ARM64.
2. **No graphics until text works.** UART console before WebGPU framebuffer.
3. **No JIT until interpreter works.** Measure first, optimize second.
4. **One test per instruction.** No untrusted code paths.

## Target

Boot a minimal ARM64 Linux kernel (vmlinuz + busybox) to an interactive shell in the browser.

## Architecture

- **CPU**: Pre-decoded threaded interpreter for AArch64.
- **Memory**: 1 GiB contiguous `Vec<u8>` (physical only until MMU needed).
- **Devices**: PL011 UART at `0x0900_0000`. GICv2 stub. VirtIO GPU later.
- **Storage**: OPFS for disk images.
- **Display**: Xterm.js for serial, HTML5 Canvas/WebGPU for framebuffer.

## License

AGPL-3.0. Commercial licensing available on request.
