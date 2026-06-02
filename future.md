# WebBoxVM — Future Work

These are aspirational targets, not committed sprints. Most depend on the Linux shell and installer milestone completing first.

## ISA Completeness
- [ ] NEON / SIMD load, store, and vector arithmetic
- [ ] Crypto extensions such as AES and SHA
- [ ] Harden ARMv8.1 LSE atomics, including memory ordering edge cases
- [ ] Remaining system instructions

## Devices and Firmware
- [ ] ACPI tables: RSDP, DSDT, MADT, GTDT, FADT
- [ ] NVMe or a more complete VirtIO storage controller
- [ ] TPM 2.0 MMIO stub
- [ ] Framebuffer through VirtIO GPU or simple linear FB
- [ ] Keyboard and mouse input through HID

## Windows 11 ARM64
- [ ] Parse Windows ISO / WIM
- [ ] Boot Windows PE to kernel loader to `ntoskrnl.exe`
- [ ] Reach desktop with display and input
- [ ] Network adapter through VirtIO Net

## WebAssembly Target
- [x] Compile to wasm32 + wasm-bindgen
- [x] Browser deployment with xterm.js console
- [x] Sparse guest memory for browser builds
- [x] Session-local sparse install disk for browser ISO boots
- [x] OPFS persistent disk for browser storage
- [x] Move to wasm64 when browser and toolchain support is practical
- [ ] Web Worker execution so long boot runs do not block the UI thread

## Backlog
- Multi-core SMP
- JIT compilation from ARM64 to native code
- x86_64 interpreter for broader OS compatibility
- Commercial licensing / dual-license
