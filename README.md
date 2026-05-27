# 🌐 WebBoxVM

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-AGPL--3.0%20%2F%20Commercial-blue.svg)](LICENSE.md)

**WebBoxVM** is a client-side, high-performance ARM64 virtual machine designed to run entirely in the browser using WebAssembly (Wasm) and WebGPU. 

Unlike traditional emulators that rely on heavy cloud infrastructures or lack modern graphics capabilities, WebBoxVM aims to bring lightweight, zero-latency virtualization directly to client browsers with zero server hosting costs.

---

## 🚀 Vision & Key Features

* **Bare-Metal ARM64 first**: Dedicated interpreting engine for AArch64 architectures before any legacy x86 support.
* **Boot real OS Kernels**: Custom PE/EFI segment parsing to boot genuine production-grade kernels (like the Debian Linux kernel).
* **High-fidelity systems modeling**: Pure-Rust simulation of registers, status words (`PState`), exception levels (`EL3`), and memory-mapped IO (MMIO).
* **Hardware-accelerated console**: Built-in support for simulated hardware serial interfaces (**PL011 UART**) with a roadmap for canvas and WebGPU-based framebuffers.
* **Permissive, clean design**: Implemented using modular systems architecture to prevent copyleft dependency pollution, making the codebase perfectly dual-licensed.

---

## 🏛️ Project Architecture

The emulator workspace is split into clean, logical segments mimicking physical machine hardware:

```
WebBoxVM/
├── emulator/              # Core emulator crate
│   ├── src/
│   │   ├── arm64/         # CPU instruction interpreter, decoder, and state registers
│   │   ├── efi/           # Minimal UEFI bootloader, runtime structures, and trampolines
│   │   ├── devices/       # Hardware device simulation (PL011 UART console, GIC stubs)
│   │   ├── bus.rs         # System MMIO memory router
│   │   ├── memory.rs      # Flat 1 GiB Physical Memory RAM simulation
│   │   ├── loader.rs      # PE-COFF kernel loader and boot preparation
│   │   └── lib.rs         # Module registry
│   └── tests/             # Workspace integration tests
├── Image.gz               # Debian Linux ARM64 kernel image
├── models.md              # Documentation of the core data structures
├── todo.md                # Development sprints and roadmap
└── vision.md              # Strategic product vision
```

---

## 🧠 Core Systems

### The ARM64 CPU Core
* **[registers.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/arm64/registers.rs)**: Manages 31 general-purpose 64-bit registers (`X0..X30`), the Stack Pointer (`SP`), and the Program Counter (`PC`).
* **[instr.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/arm64/instr.rs)**: The opcode decoder which decodes raw bytes into runnable instruction models (e.g., `ADD`, `SUB`, `MOVZ`, `LDP/STP`, conditional branches).
* **[interpreter.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/arm64/interpreter.rs)**: Runs the fetch-decode-execute instruction cycle.

### The System Motherboard & I/O
* **[memory.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/memory.rs)**: Allocates a flat 1 GiB address space as a contiguous virtual RAM block.
* **[bus.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/bus.rs)**: Dispatches read/write operations depending on target addresses, automatically directing serial outputs to the UART and relocations to physical memory.
* **[pl011.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/devices/pl011.rs)**: Mimics a standard ARM PL011 UART serial interface, redirecting output string logs to the virtual console.

### The UEFI Bootloader Stubs
* **[efi/](file:///Users/petreleon/code/WebBoxVM/emulator/src/efi/)**: Models the minimal System Table, Boot Services, and Runtime Services requested by modern OS kernels during bootup, preventing execution crashes before OS initialization completes.

---

## 🚀 Getting Started

### Prerequisites

Make sure you have a modern Rust toolchain installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Running Tests

The workspace features a comprehensive suite of 40+ unit and integration tests confirming core instruction decodes, bootloader relocations, and UART outputs:

```bash
# Run all tests
cargo test

# Run a specific test
cargo test hello_uart
```

---

## 🗺️ Development Roadmap

Development progress is organized around consecutive sprints outlined in [todo.md](file:///Users/petreleon/code/WebBoxVM/todo.md):

* **Sprint 1: CPU Core** (Complete) ✅
* **Sprint 2: Bootloader** (Complete) ✅
* **Sprint 3: EFI Stub Protocols** (Complete) ✅
* **Sprint 4: PE Relocations & Decompressor** (Complete) ✅
* **Sprint 5: MMU & TLB Walks** (Active development) 🚀
* **Sprint 6: Interactive BusyBox Shell** (Planned) 📅

---

## ⚖️ Licensing

WebBoxVM is dual-licensed under:
* **AGPL-3.0**: For open-source hobbyists and community projects.
* **Proprietary Commercial License**: Available on request for commercial and closed-source applications.

See [vision.md](file:///Users/petreleon/code/WebBoxVM/vision.md) for licensing guidelines and contact details for private licensing requests.
