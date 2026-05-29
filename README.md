# WebBoxVM

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-AGPL--3.0%20%2F%20Commercial-blue.svg)](LICENSE.md)
[![Tests](https://img.shields.io/badge/tests-98%20passed-green.svg)]()

**WebBoxVM** is an ARM64 virtual machine written in Rust. It emulates an AArch64 CPU, MMU with TLB, UEFI firmware, and essential peripherals — enough to boot a real Linux kernel from the PE/COFF entry point through the EFI stub, past the MMU setup, and into kernel virtual address space.

The emulator compiles to both native code and WebAssembly, making it suitable for browser deployment alongside native CLI testing.

---

## What works today

- **90 ARM64 opcodes** — full integer ISA, load/store pairs, exclusives, bitfield ops, multiply/divide, system registers
- **MMU** — 3-level page table walk (48-bit VA), 2048-entry software TLB, `SCTLR_EL1.M` gating
- **PL011 UART** — full register emulation (13 registers), 7 unit tests matching kernel code paths
- **GICv3** — distributor + redistributor MMIO, CPU interface via system registers, timer IRQ delivery
- **UEFI firmware** — System Table, Boot/Runtime Services, real AllocatePages bump allocator, HandleProtocol, GetMemoryMap
- **PE/COFF loader** — parses PE optional header, reads entry point dynamically, applies relocations
- **Linux boot** — PE entry → EFI stub (3.5M steps) → handoff → `primary_entry` → MMU enable → kernel VA space
- **Timer/IRQ** — WFI/WFE decode, 100 Hz tick, VBAR_EL1 delivery, DAIF mask control
- **98 tests** — zero failures, zero compiler warnings

---

## Architecture

```
emulator/src/
├── arm64/           # CPU: decode (88 opcodes), execute (~760 lines), MMU, TLB
│   ├── interpreter/ # Classic fetch-decode-execute loop
│   └── jit/         # ARM64→ARM64 verbatim compiler (skeleton)
├── efi/             # UEFI tables, trampolines, protocol stubs
├── devices/         # PL011 UART, GICv3 interrupt controller
├── loader/          # PE/COFF parser, relocation fixup
├── dtb.rs           # Device Tree Blob generator
├── initrd.rs        # cpio newc initrd builder
├── boot.rs          # Kernel boot pipeline (EFI phase → handoff → kernel phase)
├── bus.rs           # MMIO dispatch (UART, GIC, RAM)
├── memory.rs        # 3-region physical memory (low, RAM, EFI)
└── constants.rs     # Every magic number, documented
```

---

## Quick Start

```bash
# Run all tests
cargo test                          # 98 passed, 0 failed

# Build a relocatable ARM64 kernel (via Docker)
docker build -t kernel-builder .dockerbuild
docker run --rm -v $(pwd):/out kernel-builder

# Boot the kernel
cargo run --example boot_test --release
```

---

## Roadmap

| Sprint | Status |
|--------|--------|
| CPU core (90 opcodes) | ✅ |
| Bootloader + EFI firmware | ✅ |
| MMU + TLB + page tables | ✅ |
| PE loader + relocations | ✅ |
| **Busybox shell** | 🚧 in progress |
| Exception model + NEON | 📅 planned |
| Display + input | 📅 planned |
| Windows 11 ARM64 | 📅 future |

Full details in [todo.md](todo.md).

---

## License

AGPL-3.0. Commercial licensing available on request.
