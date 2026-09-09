# P13 — Profile and optimize guest CPU graphics submission

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P13
Depends: P03, I02, I03
Evidence: pending

Prerequisite lists: [P03](../../01-measurement/03-matched-baseline-runs/README.md), [I02](../../../05-guest-validation/01-real-clients/02-mesa-opengl/README.md), [I03](../../../05-guest-validation/01-real-clients/03-mesa-vulkan/README.md).

## Outcome

Measured guest CPU execution improvements reduce real graphics overhead while preserving ARM64 and
VM scheduling correctness.

## Starting points

- [web/js/vm-worker/jit-compile.js](../../../../../web/js/vm-worker/jit-compile.js)
- [web/js/vm-worker/pump.js](../../../../../web/js/vm-worker/pump.js)
- [web/js/vm-worker/jit-stats.js](../../../../../web/js/vm-worker/jit-stats.js)
- [web/js/vm-worker/state.js](../../../../../web/js/vm-worker/state.js)

## Checklist

- [ ] Profile real guest Mesa command construction, syscalls, virtqueue/MMIO and JIT
  compile/dispatch separately from GPU time.
- [ ] Choose one dominant cost with a semantic-preserving optimization such as stable block reuse or
  batching host boundary work; avoid moving general ARM64 execution into WGSL.
- [ ] Add focused ARM64/differential or scheduler regression coverage for the actual changed path
  and preserve serial/threaded behavior.
- [ ] Evaluate CPU-heavy small-draw and GPU-heavy scenes, cold/warm JIT, and interactive/network
  load under P01.
- [ ] Report complete guest performance and retain the optimization only after its benefit and
  compilation/memory costs are measured.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run `node --test web/js/vm-worker/jit-compile.test.mjs web/js/vm-worker/pump-responsive.test.mjs`
  plus the targeted existing Rust suite for any modified CPU/JIT behavior.
- Guest API output, architectural state and fence/order behavior match reference execution; JIT
  fallback reasons remain visible.
- A graphics profile blocked by CPU emulation remains blocked until full guest-to-present targets
  pass; GPU-only timing never replaces that denominator.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
