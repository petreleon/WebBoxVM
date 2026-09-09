# P02 — Instrument guest-to-present timing and resource costs

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P02
Depends: P01, R02, R06, R07
Evidence: pending

Prerequisite lists: [P01](../01-native-comparison-contract/README.md), [R02](../../../01-shared-runtime/01-resources/02-object-identity/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md), [R07](../../../01-shared-runtime/02-execution/03-scheduler/README.md).

## Outcome

A bounded correlated trace attributes complete guest frames, readbacks, queues and resource costs
without treating queue completion as physical presentation.

## Starting points

- [emulator/src/host/wasm/gpu_api.rs](../../../../../emulator/src/host/wasm/gpu_api.rs)
- [web/js/vm-worker/gpu-3d.js](../../../../../web/js/vm-worker/gpu-3d.js)
- [web/js/gpu-display.js](../../../../../web/js/gpu-display.js)
- [web/js/gpu-display-diagnostics.js](../../../../../web/js/gpu-display-diagnostics.js)

## Checklist

- [ ] Assign trace identities across guest submission, Rust decode, Wasm packet export, worker
  send/receive, browser encode/submit, GPU completion, fence acknowledgement and presented-frame
  observation.
- [ ] Calibrate clocks across guest, worker and browser with recorded uncertainty; separate
  observable browser presentation timing from externally measured input-to-photon when available.
- [ ] Add bounded counters for bytes copied/uploaded/read back, live/peak resources, cache hits,
  queue depth, fence waits, dropped work, errors and fallback reason.
- [ ] Capture GPU timestamp queries only where supported and record their availability; keep tracing
  opt-in with fixed bounds and aggregate sampling.
- [ ] Add verification for sequence correlation, reset generation changes, asynchronous completion,
  missing timestamps and instrumentation overhead.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run `node --test web/js/vm-worker/gpu-3d.test.mjs web/js/gpu-display-3d.test.mjs
  web/js/gpu-display-reset.test.mjs`; changed boundaries retain existing completion/reset behavior.
- A trace of one real guest API frame contains correlated start, completion and presentation
  observations, while missing metrics remain unavailable rather than zero.
- Measure tracing enabled versus disabled on the frozen workload and report overhead against the P01
  measurement budget.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
