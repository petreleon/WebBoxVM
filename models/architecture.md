# Modular Architecture Goal

This is the active architecture target for WebBoxVM. The purpose is not just
smaller files; the purpose is strict ownership boundaries that make illegal
cross-layer dependencies hard to introduce.

## Target Layout

```text
emulator/src/
  api/              # public facade: VmHandle, VmConfig, VmEvent
  runtime/          # run loop, scheduling, clocks, tracing, step budgets
  arch/
    arm64/          # CPU state, decode, execute, MMU, sysregs, exceptions
  memory/           # sparse physical memory, DMA access traits
  platform/
    virt/           # board config, physical memory map, bus, IRQ wiring
  devices/          # PL011, GICv3, VirtIO, each isolated behind MMIO
  boot/             # BootPlan builder: kernel/initrd/dtb/register init
  images/           # ISO/kernel parsers; pure bytes in, boot artifacts out
  host/
    wasm/           # wasm-bindgen adapter only
    native/         # CLI/examples adapter only
  observability/    # trace sinks, counters, debug snapshots
```

## Boundary Rules

1. `arch::arm64` owns CPU semantics only.
2. `runtime` owns stepping, scheduling, clocks, and trace dispatch.
3. `platform::virt` owns physical address layout, MMIO routing, and IRQ wiring.
4. `devices` never know about `Machine`, wasm, boot plans, or UI state.
5. `boot` produces a `BootPlan`; it does not own a live VM.
6. `host::wasm` and `host::native` adapt public APIs; they do not reach into
   CPU, bus, memory, or device internals directly.
7. Bus APIs accept physical addresses only. ARM64 virtual translation stays in
   the ARM64 MMU.

## File Size Rule

All maintained source files stay at or below 180 lines. Legal text, generated
artifacts, large binary assets, and lockfiles are exempt.

## Test Policy

Every migration slice needs focused tests before or with the move:

- Boundary tests for public APIs and forbidden ownership leaks.
- Invariant tests for mutable state such as buses, devices, TLBs, and queues.
- Regression tests for boot-sensitive CPU, MMU, timer, UART, GIC, and VirtIO
  behavior.
- Property-style tests where examples are too narrow.

## First Migration Slices

1. Add typed boundary primitives: physical addresses, virtual addresses, IRQ IDs,
   and access widths.
2. Move `Machine` orchestration from `arm64` into `runtime`.
3. Split the current concrete `SystemBus` into `platform::virt` routing plus
   isolated device implementations.
4. Replace `BootContext` ownership with a pure `BootPlan`.
5. Move wasm state into a host adapter around the public VM facade.
