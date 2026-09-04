# WebBoxVM

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-AGPL--3.0%20%2F%20Commercial-blue.svg)](LICENSE.md)
[![Boot](https://img.shields.io/badge/boot-Debian%20installer-green.svg)]()

**WebBoxVM** is an ARM64 virtual machine written in Rust. It emulates an AArch64 CPU, MMU with TLB, interrupt/timer state, a PL011 UART, a GICv3 interrupt controller, and enough platform devices to boot real ARM64 Linux media to a serial terminal.

The emulator compiles to both native code and wasm64 WebAssembly, making it suitable for Memory64-capable browser deployment alongside native CLI testing.

## What works today

- **ARM64 CPU core** — integer ISA coverage, load/store pairs, exclusives, LSE atomics, conditional compares, bitfield ops, multiply/divide, system registers
- **MMU** — 3-level page table walk (48-bit VA), 2048-entry software TLB, `SCTLR_EL1.M` gating
- **PL011 UART** — full register emulation (13 registers), RX interrupt delivery, and unit tests matching kernel code paths
- **GICv3 + timer IRQs** — distributor + redistributor MMIO, CPU interface sysregs, CNTP control/status, DAIF masking, current-EL vector delivery
- **Device tree + initrd** — RAM, CPU, timer, GIC, UART, chosen bootargs, and minimal cpio initrd generation
- **BusyBox initrd + serial input** — embedded static ARM64 BusyBox, `/init`, `/dev/console`, applet symlinks, and UART RX wiring for shell input
- **ARM64 ISO terminal boot path** — extracts kernel/initrd from ISO9660 media, attaches the ISO as a read-only VirtIO block device, and boots through the serial terminal path
- **Persistent install storage** — exposes a second VirtIO block device backed by a sparse writable disk, with browser OPFS save/restore
- **VirtIO network path** — exposes a VirtIO-net MMIO device, browser WebSocket Ethernet hub, and Linux TAP-backed NAT peer
- **VirtIO-GPU 2D scanout** — Linux can create, back, transfer, and flush a fixed 1024×768 scanout in CPU-side Rust/Wasm; WebGPU only presents dirty BGRA rectangles on the host, with Canvas2D used when startup WebGPU initialization is unavailable
- **Guest 3D paths** — [limited standard VirGL capsets 1 and 2](research/virgl-compatibility.md) validate resource creation, transfer/readback, attached off-screen copy, surface/framebuffer binding, color clear, exact source-over blending, and bounded normalized-TGSI solid, fragment-inline-constant, or resource-backed-fragment-constant, canonical four-row `DP4` vertex-matrix with fixed generic passthrough, interpolated or constant-modulated vertex-color, sampled or fragment-constant-modulated texture, texture-times-vertex-color, or independently sampled two-texture `DRAW_VBO` paths. Three bounded non-depth DP4 lanes send raw vertices plus their row-major matrix to a browser WebGPU shader: solid, exact generic vertex-color without fragment-constant multiplication, and exact generic-UV one-texture sampling without modulation. CPU replay remains the guest-memory correctness path. The routes resolve fixed u16/u32 indexes plus one-to-three standard VBO sources and normalize `TRIANGLES`, alternating-winding `TRIANGLE_STRIP`, or spoke-preserving `TRIANGLE_FAN` input to bounded lists, with rasterizer, viewport, scissor, and canonical DSA depth compare/write-mask state; `VGM1`/`VGB1` passes preserve 2–16 ordered snapshots and can return mapped GPU color through guest completion. Private capset 7 carries separate bounded `WBG3` geometry. Neither route is Mesa/OpenGL or Vulkan/Venus compatibility yet.
- **Debian installer milestone** — Debian ARM64 netinst reaches the text installer in browser validation, loads installer components from ISO media to 100%, and advances to network hardware detection
- **Browser terminal app** — wasm64 worker build with xterm.js console, ISO picker, Debian boot target, persistent disk controls, UART keyboard input, and live VM metrics
- **Parallel vCPU execution** — multicore native boots use one host thread per vCPU; isolated browsers use a persistent Web Worker per vCPU over one shared Memory64 heap
- **Experimental staged multicore boot** — a guarded path for compatible Debian/systemd saved systems defers CPU1 through the critical path, verifies late hotplug, then switches to parallel vCPU workers; exact-fixture browser validation is still pending
- **Sparse guest memory** — guest RAM/low/EFI regions allocate touched 4 KiB pages instead of reserving the full platform address layout up front
- **Conservative browser JIT** — Wasm64 basic-block JIT for safe paths, with EL0 guest-memory helper blocks skipped when helper-call overhead or speculative memory effects would hurt progress
- **UEFI/PE infrastructure** — System Table, Boot/Runtime Services, PE header parsing, and relocation helpers remain available for EFI experiments
- **Linux early UART boot** — standard ARM64 Image protocol → `primary_entry` → MMU enable → kernel VA space → early PL011 console output
- **Regression coverage** — focused tests for Linux boot-sensitive instruction semantics, timer IRQ behavior, UART, MMU, loader, and device paths

## Architecture

```
emulator/src/
├── arm64/           # CPU decode/execute, system registers, MMU, TLB
│   ├── interpreter/ # Classic fetch-decode-execute loop
│   └── jit/         # Native experiments and conservative ARM64→Wasm64 blocks
├── efi/             # UEFI tables, trampolines, protocol stubs
├── devices/         # PL011, GICv3, and VirtIO block, network, and GPU devices
├── loader/          # PE/COFF parser, relocation fixup
├── dtb.rs           # Device Tree Blob generator
├── initrd/          # cpio newc initrd parser and builder
├── boot/            # Standard ARM64 Linux Image boot pipeline
├── bus.rs           # MMIO dispatch (UART, GIC, RAM)
├── memory.rs        # Sparse 3-region physical memory (low, RAM, EFI)
└── constants.rs     # Every magic number, documented
```

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

# Build the wasm64 package and open the browser terminal app
make web

# Download Debian, expose it to the browser app, and serve WebBoxVM
make web-debian-arm64

make web-benchmark
# Open http://localhost:8080/?benchmark=installed-disk

# On a Linux NAT peer, route browser VM Ethernet through the host network
sudo python3 scripts/webbox_nat.py --configure-host
```

`make web` and `make web-debian-arm64` build two `wasm64-unknown-unknown` packages with nightly `build-std`: `web/pkg/` is the serial fallback and
`web/pkg-threaded/` uses atomics and shared Memory64. Both packages are generated
with the same revision-pinned, Memory64-aware wasm-bindgen tool built by
`make wasm-bindgen-memory64-threads`. Generated WASM output is not committed to the repository.

Threaded browser execution requires Memory64, `SharedArrayBuffer`, Web Workers,
and cross-origin isolation. The bundled server sends the required COOP/COEP
headers and disables caching. Cacheable deployments must keep the stamped ESM
graph generation together or use content-hashed bundles. Other deployments must
send COOP/COEP; otherwise the loader uses the serial package. ISO boots enter parallel workers immediately.
An experimental two-core saved-disk boot path stages CPU1 only after exact bootarg, root/systemd, selected-kernel hotplug, initramfs, and worker-preflight checks pass. If worker
preflight is unavailable, both vCPUs boot cooperatively and staging is never
requested. If the guest capability gate declines with workers ready, both CPUs
boot directly in parallel. Neither fallback adds `maxcpus=1`. The compatible
path runs CPU0 with the Wasm64 JIT, hot-plugs CPU1 immediately before serial login, then switches to parallel Wasm after both milestones.
Unit and native preparation checks cover this flow, but an exact-fixture browser run has not yet completed both milestones.

Installed-disk startup is a firmware-level fast path for ARM64, not Intel Fast
Boot: WebBoxVM validates the sparse disk and installed kernel/initrd, builds the
DTB, and enters the standard ARM64 Linux Image protocol directly. It executes
zero EFI/firmware guest instructions, so there is no PC-style POST, DXE/BDS, or
device-enumeration phase to skip.

`make web-benchmark` maps `output/webboxvm-final-install-compact.wbdisk` to a fixed same-origin URL
after verifying its SHA-256. Benchmark mode validates the fixed response length
and WBDISK structure, disables OPFS and autosave, and reports host-time firmware,
CPU1-online, login, and execution-mode milestones in the event log.

The browser network path uses `/webboxvm-net` for raw Ethernet frames. See
[scripts/networking.md](scripts/networking.md) for the Linux TAP/NAT peer setup.

ISO mode supports ARM64 Linux ISOs whose kernel/initrd can be discovered from
GRUB config or common live/installer paths. It does not run x86 PC ISOs. Debian
ARM64 netinst has been validated in the browser through language, location, and
keymap selection, installation-media scan, installer-component loading to 100%,
and the next "Detecting network hardware" step. The ISO is exposed to the guest
as read-only media, and WebBoxVM also provides a second writable sparse VirtIO
disk for installer storage. Browser disk contents are saved to Origin Private
File System storage as compact sparse-disk snapshots and restored on the next
boot from the same origin.

The browser app runs the wasm64 VM in a module Web Worker, renders xterm.js and a WebGPU display,
wires UART input, and persists its sparse install disk through OPFS.

Successful early boot prints lines like:

```text
[    0.000000] Linux version 6.6.70 ...
[    0.000000] Machine model: WebBoxVM
[    0.000000] earlycon: pl11 at MMIO 0x0000000009000000
```

The longer Debian ISO validation reaches:

```text
Loading additional components ... 100%
Detecting network hardware ... 100%
```

## Roadmap

| Sprint | Status |
|--------|--------|
| CPU core | ✅ |
| Bootloader + EFI firmware | ✅ |
| MMU + TLB + page tables | ✅ |
| PE loader + relocations | ✅ |
| **Linux early UART boot** | ✅ |
| **BusyBox shell** | ✅ |
| **ARM64 Debian installer over serial** | ✅ |
| Browser xterm.js terminal | ✅ |
| Wasm64 browser target | ✅ |
| Browser worker execution | ✅ |
| Debian component loading in browser | ✅ |
| Browser network + NAT path | ✅ initial |
| Exception model + NEON | 📅 planned |
| Display + input | 🧪 2D scanout; cursor and HID planned |
| Windows 11 ARM64 | 📅 future |

Full details in [todo.md](todo.md), [sprint-history.md](sprint-history.md), and [future.md](future.md).

## License

AGPL-3.0. Commercial licensing available on request.
