# WebBoxVM

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-AGPL--3.0%20%2F%20Commercial-blue.svg)](LICENSE.md)
[![Boot](https://img.shields.io/badge/boot-Debian%20installer-green.svg)]()

**WebBoxVM** is an ARM64 virtual machine written in Rust. It emulates an AArch64 CPU, MMU with TLB, interrupt/timer state, a PL011 UART, a GICv3 interrupt controller, and enough platform devices to boot real ARM64 Linux media to a serial terminal.

The emulator compiles to both native code and WebAssembly, making it suitable for browser deployment alongside native CLI testing.

---

## What works today

- **ARM64 CPU core** — integer ISA coverage, load/store pairs, exclusives, LSE atomics, conditional compares, bitfield ops, multiply/divide, system registers
- **MMU** — 3-level page table walk (48-bit VA), 2048-entry software TLB, `SCTLR_EL1.M` gating
- **PL011 UART** — full register emulation (13 registers), RX interrupt delivery, and unit tests matching kernel code paths
- **GICv3 + timer IRQs** — distributor + redistributor MMIO, CPU interface sysregs, CNTP control/status, DAIF masking, current-EL vector delivery
- **Device tree + initrd** — RAM, CPU, timer, GIC, UART, chosen bootargs, and minimal cpio initrd generation
- **BusyBox initrd + serial input** — embedded static ARM64 BusyBox, `/init`, `/dev/console`, applet symlinks, and UART RX wiring for shell input
- **ARM64 ISO terminal boot path** — extracts kernel/initrd from ISO9660 media, attaches the ISO as a read-only VirtIO block device, and boots through the serial terminal path
- **Writable install storage** — exposes a second VirtIO block device backed by a sparse writable disk for Linux installer targets
- **Debian installer milestone** — Debian ARM64 netinst reaches the real text installer language prompt in native CLI validation
- **Browser terminal app** — WASM build with xterm.js console, ISO picker, Debian boot target, disk-size control, UART keyboard input, and live VM metrics
- **Sparse guest memory** — guest RAM/low/EFI regions allocate touched 4 KiB pages instead of reserving the full platform address layout up front
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
├── devices/         # PL011 UART, GICv3 interrupt controller, VirtIO block storage
├── loader/          # PE/COFF parser, relocation fixup
├── dtb.rs           # Device Tree Blob generator
├── initrd/          # cpio newc initrd parser and builder
├── boot/            # Standard ARM64 Linux Image boot pipeline
├── bus.rs           # MMIO dispatch (UART, GIC, RAM)
├── memory.rs        # Sparse 3-region physical memory (low, RAM, EFI)
└── constants.rs     # Every magic number, documented
```

---

## Quick Start

```bash
# Run the emulator test suite
cargo test -p emulator

# Build a relocatable ARM64 kernel (via Docker)
docker build -t kernel-builder .dockerbuild
mkdir -p .artifacts
docker run --rm -v $(pwd)/.artifacts:/out kernel-builder

# Refresh the local ARM64 BusyBox used by the default initrd
scripts/update_busybox.sh

# Download the current Debian ARM64 netinst ISO with checksum verification
make iso-debian-arm64

# Inspect the kernel/initrd that WebBoxVM will extract from that ISO
make iso-info

# Boot until Linux writes to the PL011 UART
cargo run --example wait_uart --release

# Run the interactive serial terminal frontend
cargo run -p emulator --example terminal --release -- .artifacts/Image

# Or extract and boot the downloaded ARM64 ISO through the serial terminal
make terminal-debian-arm64

# Boot any other ARM64 Linux ISO through the serial terminal
make terminal-iso ISO=path/to/arm64.iso

# Build the WASM package and open the browser terminal app
make web

# Download Debian, expose it to the browser app, and serve WebBoxVM
make web-debian-arm64
```

`make web` and `make web-debian-arm64` build `web/pkg/` on demand with
`wasm-bindgen`; generated WASM output is not committed to the repository.

ISO mode supports ARM64 Linux ISOs whose kernel/initrd can be discovered from
GRUB config or common live/installer paths. It does not run x86 PC ISOs. Debian
ARM64 netinst has been validated to reach the text installer language prompt.
The ISO is exposed to the guest as read-only media, and WebBoxVM also provides a
second writable sparse VirtIO disk for installer storage. Browser disk contents
are currently in-memory for the VM session; OPFS persistence is tracked as a
separate milestone.

The native terminal path is currently the most validated path. The browser app
loads, renders the xterm.js terminal, exposes ISO boot and install-disk controls,
and wires UART keyboard input, but a full Debian-to-installer browser run is
still the next validation target.

Successful early boot prints lines like:

```text
[    0.000000] Linux version 6.6.70 ...
[    0.000000] Machine model: WebBoxVM
[    0.000000] earlycon: pl11 at MMIO 0x0000000009000000
```

The longer Debian ISO validation reaches:

```text
Choose the language to be used for the installation process.
Language:
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
| **ARM64 Debian installer over serial** | 🚧 in progress |
| Browser xterm.js terminal | 🚧 in progress |
| Exception model + NEON | 📅 planned |
| Display + input | 📅 planned |
| Windows 11 ARM64 | 📅 future |

Full details in [todo.md](todo.md).

---

## License

AGPL-3.0. Commercial licensing available on request.
