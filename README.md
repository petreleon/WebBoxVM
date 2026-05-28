# 🌐 WebBoxVM

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-AGPL--3.0%20%2F%20Commercial-blue.svg)](LICENSE.md)

**WebBoxVM** is a client-side **Windows 11 ARM64** virtual machine designed to run entirely in the browser using WebAssembly (Wasm) and WebGPU.

The end goal is booting a retail Windows 11 ARM64 ISO to the desktop inside any browser tab — zero server cost, zero latency, native ARM64 app compatibility on Snapdragon X Elite and Apple Silicon laptops.

Linux + Busybox is the current stepping stone (Sprints 1–6). Windows 11 is the destination (Phase 2).

---

## 🚀 Vision & Key Features

* **Windows 11 ARM64 in the browser**: The main objective. Linux + Busybox is the proof-of-concept stepping stone.
* **Bare-Metal ARM64 first**: Dedicated interpreting engine for AArch64 architectures before any legacy x86 support.
* **Boot real OS Kernels**: Custom PE/EFI segment parsing to boot genuine production-grade kernels (Debian Linux today, Windows 11 tomorrow).
* **High-fidelity systems modeling**: Pure-Rust simulation of registers, status words (`PState`), exception levels (`EL3`), MMU/TLB, and memory-mapped IO (MMIO).
* **Hardware-accelerated console**: Built-in support for simulated hardware serial interfaces (**PL011 UART**) with a roadmap for canvas and WebGPU-based framebuffers.
* **Permissive, clean design**: Implemented using modular systems architecture to prevent copyleft dependency pollution, making the codebase perfectly dual-licensed.

---

## 🏛️ Project Architecture

The emulator workspace is split into clean, logical segments mimicking physical machine hardware:

```
WebBoxVM/
├── emulator/              # Core emulator crate
│   ├── src/
│   │   ├── arm64/         # CPU instruction interpreter, decoder, execute, and state registers
│   │   │   ├── mmu.rs     # MMU: 3-level page table walk + 2048-entry software TLB
│   │   │   ├── opcodes.rs # Instruction opcodes and decoded representation
│   │   │   ├── pstate.rs  # Processor state (NZCV flags, exception level)
│   │   │   ├── system_regs.rs  # System register file (TTBR, TCR, SCTLR, VBAR, etc.)
│   │   │   ├── decode.rs  # AArch64 instruction decoder
│   │   │   ├── execute.rs # Instruction execution engine
│   │   │   ├── helpers.rs # Register read/write helpers, condition codes
│   │   │   ├── bitmask_imm.rs  # Bitmask immediate decoder
│   │   │   └── interpreter/ # Fetch-decode-execute loop
│   │   ├── efi/           # Minimal UEFI bootloader, runtime structures, and trampolines
│   │   ├── devices/       # Hardware device simulation (PL011 UART, GICv3, TPM 2.0)
│   │   ├── initrd/        # cpio newc initrd builder and loader
│   │   ├── dtb.rs         # Device Tree Blob generator
│   │   ├── loader/        # PE-COFF kernel loader, relocations, and boot preparation
│   │   ├── bus.rs         # System MMIO memory router
│   │   ├── memory.rs      # Flat physical memory with RAM + EFI regions
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
* **[decode.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/arm64/decode.rs)**: The opcode decoder which decodes raw 32-bit ARM64 words into runnable instruction models (e.g., `ADD`, `SUB`, `MOVZ`, `LDP/STP`, conditional branches, `TLBI`).
* **[execute.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/arm64/execute.rs)**: The execution engine that mutates CPU and bus state per decoded instruction.
* **[interpreter/](file:///Users/petreleon/code/WebBoxVM/emulator/src/arm64/interpreter/)**: Runs the fetch-decode-execute instruction cycle with MMU-aware PC translation.

### Memory Management Unit (MMU)
* **[mmu.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/arm64/mmu.rs)**: Implements a 3-level page table walk for 39-bit virtual addresses (4 KB granule), a 2048-entry software TLB, and `SCTLR_EL1.M` gating. Supports 4 KB pages, 2 MB blocks, and 1 GB blocks.

### The System Motherboard & I/O
* **[memory.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/memory.rs)**: Simulates three disjoint physical regions (low, RAM, EFI) with flat byte backing.
* **[bus.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/bus.rs)**: Dispatches read/write operations depending on target addresses, automatically directing serial outputs to the UART and relocations to physical memory.
* **[pl011.rs](file:///Users/petreleon/code/WebBoxVM/emulator/src/devices/pl011.rs)**: Mimics a standard ARM PL011 UART serial interface, redirecting output string logs to the virtual console.

### The UEFI Bootloader Stubs
* **[efi/](file:///Users/petreleon/code/WebBoxVM/emulator/src/efi/)**: Models the minimal System Table, Boot Services, and Runtime Services requested by modern OS kernels during bootup, preventing execution crashes before OS initialization completes. PE relocations, `AllocatePages`, `GetMemoryMap`, and `ExitBootServices` are all implemented.

---

## 🚀 Getting Started

### Prerequisites

Make sure you have a modern Rust toolchain installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Running Tests

The workspace features a comprehensive suite of 97 unit and integration tests confirming core instruction decodes, bootloader relocations, MMU page table walks, TLB behavior, and UART outputs:

```bash
# Run all tests
cargo test

# Run a specific test
cargo test hello_uart
```

---

## 🗺️ Development Roadmap

Development progress is organized around consecutive sprints outlined in [todo.md](file:///Users/petreleon/code/WebBoxVM/todo.md):

### Phase 1 — Linux Proof of Concept

* **Sprint 1: CPU Core** (Complete) ✅
* **Sprint 2: Bootloader** (Complete) ✅
* **Sprint 3: EFI Stub Protocols** (Complete) ✅
* **Sprint 4: PE Relocations & Decompressor** (Complete) ✅
* **Sprint 5: MMU & TLB Walks** (Complete) ✅
* **Sprint 6: Interactive BusyBox Shell** (Active — blocked on BRK #0x800 crash) 🚀

### Phase 2 — Windows 11 ARM64

* **Sprint 7: Exception Model & Interrupts** (Planned) 📅
* **Sprint 8: ISA Completeness (NEON, Crypto, Atomics)** (Planned) 📅
* **Sprint 9: ACPI & Firmware Tables** (Planned) 📅
* **Sprint 10: Storage & Windows Boot** (Planned) 📅
* **Sprint 11: Windows Kernel Bring-up** (Planned) 📅
* **Sprint 12: Display & Input** (Planned) 📅
* **Sprint 13: Windows 11 Desktop** (Planned) 📅

---

## ⚖️ Licensing

WebBoxVM is dual-licensed under:
* **AGPL-3.0**: For open-source hobbyists and community projects.
* **Proprietary Commercial License**: Available on request for commercial and closed-source applications.

See [vision.md](file:///Users/petreleon/code/WebBoxVM/vision.md) for licensing guidelines and contact details for private licensing requests.
