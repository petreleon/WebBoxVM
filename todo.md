# WebBoxVM — Active Todo

Completed sprint history is mirrored in [sprint-history.md](sprint-history.md). Recent completed sprints stay here as working context; aspirational backlog lives in [future.md](future.md).

## Active Goal — Modular Architecture Boundary Refactor
- [x] Capture the target module layout and hard boundary rules in [models/architecture.md](models/architecture.md)
- [x] Enforce the 180-line source-file ceiling with an integration test
- [x] Introduce typed boundary primitives for addresses, IRQs, and access widths
- [x] Move VM runtime orchestration out of `arm64`
- [x] Split platform routing from isolated device implementations
- [x] Replace `BootContext` ownership with a pure boot plan
- [x] Move the wasm host adapter behind `host::wasm`
- [x] Keep completed migration slices covered by focused invariant and regression tests
- [x] Move ARM64 implementation under `arch::arm64` with a compatibility re-export
- [x] Move ISO/kernel loaders under `images`
- [x] Add a `host::native` adapter for CLI/examples
- [x] Extract trace sinks, counters, and debug snapshots into `observability`
- [x] Back the public `api::VmHandle` facade with runtime-owned construction
- [x] Audit legacy compatibility shims (`arm64`, `bus`, `loader`) for eventual removal

## Current Focus — Pause, Learn, Then Continue
- [ ] Read and explain the browser JIT policy in `web/js/vm-worker/jit-compile.js`
  - Why EL0 guest-memory helper blocks are skipped
  - Why a JIT block can be semantically safe but slower than interpreter fallback
- [ ] Trace the opcode telemetry path for `0x6e20ac00`
  - `disarm64` mnemonic: `uminp`
  - WebBoxVM opcode: `Opcode::SimdUminp`
  - Display name source: `emulator/src/arch/arm64/opcodes/names_simd.rs`
- [ ] Continue Debian installer validation from the next real blocker after network hardware detection
- [ ] Investigate network-device support and expected Debian installer behavior with no NIC
- [ ] Profile the userland-heavy component loading path without relaxing correctness guards

## Sprint 11 — Debian Installer Component Loading
- [x] Reproduce the browser-side installer path through language, location, and keymap selection
- [x] Identify the browser Wasm64 JIT regression from EL0 guest-memory helper blocks
- [x] Restore conservative EL0 helper-block skipping while keeping helper correctness fixes
- [x] Use `disarm64`-backed decoding to classify observed raw opcode `0x6e20ac00`
- [x] Split SIMD opcode display names into `names_simd.rs`
- [x] Verify Debian ARM64 netinst loads installer components to 100% in the browser
- [x] Verify the installer advances to `Detecting network hardware`

**Result:** The previous Debian installer failure at "Load installer components
from installation media" is no longer the active blocker. Browser validation
loaded components to 100% and advanced to network hardware detection. The next
work is network/device behavior and performance, not blind opcode expansion.

## Sprint 7 — Serial Linux Userspace and ISO Installer
- [x] Kernel boots to BusyBox `ash` shell
  - Early console works; next target is enough init, scheduler, device, and initrd behavior to spawn `/init`
  - Default initrd contains real static ARM64 BusyBox, `/init`, `/dev/console`, and applet symlinks
- [x] Replace placeholder BusyBox payload with a real static ARM64 BusyBox binary
- [x] Continue boot beyond early console into initramfs unpacking and `/init`
- [x] Add UART RX path wiring for interactive shell input
- [x] Add first ARM64 ISO terminal boot path by extracting kernel/initrd from ISO9660 media
- [x] Attach booted ISO media as a read-only VirtIO block device
- [x] Add second writable sparse VirtIO disk for installer target storage
- [x] Add sparse install-disk snapshot/restore format
- [x] Debian ARM64 netinst reaches the serial text installer language prompt
- [x] Add sparse physical memory so browser builds do not allocate the full guest address layout up front
- [x] Validate standard Debian ARM64 netinst native boot through `/lib/debian-installer/menu` and `/usr/bin/main-menu`
- [x] Standard boot for `CONFIG_RELOCATABLE=n` kernels
  - [x] Enter the ARM64 Image header at the physical load address with MMU disabled
  - [x] Pass the DTB in `X0` and clear `X1`-`X3` per the standard boot protocol
  - [x] Keep the production EFI phase out of the handoff, avoiding `_text == *image_addr` relocation checks
  - [x] Cover the non-relocatable handoff with a regression test
  - [x] Support pre-built distro ARM64 Images without Docker rebuild
- [x] Interactive commands: `ls`, `echo hello`, `cat /proc/cpuinfo`

**Result:** Native CLI boots Linux to the default BusyBox `ash` prompt, and `echo hello`, `ls /`, and `cat /proc/cpuinfo` work over the serial console. Debian ARM64 netinst reaches the real text installer language prompt. Standard ARM64 Image handoff covers non-relocatable kernels without requiring EFI relocation.

## Sprint 8 — Browser Terminal Delivery
- [x] Build `wasm32-unknown-unknown` package with `wasm-bindgen`
- [x] Add browser app shell in `web/`
- [x] Render xterm.js serial terminal
- [x] Add ISO picker and Debian boot button
- [x] Wire browser keyboard input to the guest PL011 UART receive path
- [x] Add pause, resume, reset, step-slice, and live VM metrics
- [x] Add `make web`, `make web-pkg`, and `make web-debian-arm64`
- [x] Verify the browser page loads and terminal DOM renders
- [x] Expose browser install-disk size control and sparse disk allocation metric
- [x] Persist browser install disk through OPFS snapshots with autosave and manual save/clear controls
- [x] Run Debian ARM64 netinst to the installer language prompt inside the browser app
- [x] Verify interactive browser input at the Debian prompt
- [x] Improve browser runtime speed enough for practical terminal interaction
- [x] Keep generated `web/pkg/` reproducible and uncommitted

## Sprint 9 — Wasm64 Browser Target
- [x] Make `wasm64-unknown-unknown` the default browser build target
- [x] Build the browser package through nightly `build-std`
- [x] Generate `wasm-bindgen` web glue from the wasm64 module
- [x] Add browser-side Memory64 capability detection before VM boot
- [x] Verify the wasm64 package instantiates and accepts byte-array boot input
- [x] Keep generated `web/pkg/` reproducible and uncommitted

**Result:** WebBoxVM is now wasm64-first for browser builds. `make web-pkg` builds `wasm64-unknown-unknown` with nightly `build-std`, emits browser `wasm-bindgen` glue, and the app gates boot on WebAssembly Memory64 support instead of falling back to wasm32.

## Sprint 10 — Browser Worker Execution
- [x] Move the wasm64 `Emulator` instance into a module Web Worker
- [x] Add a main-thread VM proxy with cached metrics for the existing UI
- [x] Run the kernel pump inside the worker instead of `requestAnimationFrame`
- [x] Route UART input, UART output, live metrics, and errors across the worker boundary
- [x] Preserve OPFS install-disk restore, snapshot, manual save, and autosave controls
- [x] Keep generated `web/pkg/` reproducible and uncommitted

**Result:** Browser execution no longer runs kernel steps on the UI thread. The main page owns terminal rendering and controls, while a wasm64 module worker owns the VM, boot, UART RX/TX, metrics, and disk snapshots.
