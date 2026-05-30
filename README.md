# WebBoxVM

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-AGPL--3.0%20%2F%20Commercial-blue.svg)](LICENSE.md)
[![Boot](https://img.shields.io/badge/boot-early%20UART-green.svg)]()

**WebBoxVM** is an ARM64 virtual machine written in Rust. It emulates an AArch64 CPU, MMU with TLB, interrupt/timer state, a PL011 UART, a GICv3 interrupt controller, and enough platform firmware data to boot a real Linux kernel to early serial output.

The emulator compiles to both native code and WebAssembly, making it suitable for browser deployment alongside native CLI testing.

---

## What works today

- **ARM64 CPU core** — integer ISA coverage, load/store pairs, exclusives, LSE atomics, conditional compares, bitfield ops, multiply/divide, system registers
- **MMU** — 3-level page table walk (48-bit VA), 2048-entry software TLB, `SCTLR_EL1.M` gating
- **PL011 UART** — full register emulation (13 registers), 7 unit tests matching kernel code paths
- **GICv3 + timer IRQs** — distributor + redistributor MMIO, CPU interface sysregs, CNTP control/status, DAIF masking, current-EL vector delivery
- **Device tree + initrd** — RAM, CPU, timer, GIC, UART, chosen bootargs, and minimal cpio initrd generation
- **UEFI/PE infrastructure** — System Table, Boot/Runtime Services, PE header parsing, and relocation helpers remain available for EFI experiments
- **Linux early UART boot** — standard ARM64 Image protocol → `primary_entry` → MMU enable → kernel VA space → early PL011 console output
- **Regression coverage** — focused tests for Linux boot-sensitive instruction semantics, timer IRQ behavior, UART, MMU, loader, and device paths

---

## Architecture

```
emulator/src/
├── arm64/           # CPU decode/execute, system registers, MMU, TLB
│   ├── interpreter/ # Classic fetch-decode-execute loop
│   └── jit/         # ARM64→ARM64 verbatim compiler (skeleton)
├── efi/             # UEFI tables, trampolines, protocol stubs
├── devices/         # PL011 UART, GICv3 interrupt controller
├── loader/          # PE/COFF parser, relocation fixup
├── dtb.rs           # Device Tree Blob generator
├── initrd.rs        # cpio newc initrd builder
├── boot/            # Standard ARM64 Linux Image boot pipeline
├── bus.rs           # MMIO dispatch (UART, GIC, RAM)
├── memory.rs        # 3-region physical memory (low, RAM, EFI)
└── constants.rs     # Every magic number, documented
```

---

## Quick Start

```bash
# Run the emulator test suite
cargo test -p emulator

# Build a relocatable ARM64 kernel (via Docker)
docker build -t kernel-builder .dockerbuild
docker run --rm -v $(pwd):/out kernel-builder

# Boot until Linux writes to the PL011 UART
cargo run --example wait_uart --release
```

Successful early boot prints lines like:

```text
[    0.000000] Linux version 6.6.70 ...
[    0.000000] Machine model: WebBoxVM
[    0.000000] earlycon: pl11 at MMIO 0x0000000009000000
```

---

## Roadmap

| Sprint | Status |
|--------|--------|
| CPU core | ✅ |
| Bootloader + EFI firmware | ✅ |
| MMU + TLB + page tables | ✅ |
| PE loader + relocations | ✅ |
| **Linux early UART boot** | ✅ |
| **Busybox shell** | 🚧 in progress |
| Exception model + NEON | 📅 planned |
| Display + input | 📅 planned |
| Windows 11 ARM64 | 📅 future |

Full details in [todo.md](todo.md).

---

## License

AGPL-3.0. Commercial licensing available on request.
