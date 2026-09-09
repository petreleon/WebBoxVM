# P05 — Experiment with a bounded worker transfer ring

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P05
Depends: P03, R02, R05, R07
Evidence: pending

Prerequisite lists: [P03](../../01-measurement/03-matched-baseline-runs/README.md), [R02](../../../01-shared-runtime/01-resources/02-object-identity/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md), [R07](../../../01-shared-runtime/02-execution/03-scheduler/README.md).

## Outcome

The transport reduces measured copies without weakening ownership or requiring shared memory on
unsupported browser configurations.

## Starting points

- [emulator/src/host/wasm/gpu_api.rs](../../../../../emulator/src/host/wasm/gpu_api.rs)
- [web/js/vm-worker/gpu-3d.js](../../../../../web/js/vm-worker/gpu-3d.js)
- [web/js/vm-worker/gpu-3d.test.mjs](../../../../../web/js/vm-worker/gpu-3d.test.mjs)
- [web/js/webgpu-3d-resources.js](../../../../../web/js/webgpu-3d-resources.js)

## Checklist

- [ ] Account current Rust Vec/Wasm export, sliced view, transferable ArrayBuffer and upload copies
  per frame.
- [ ] Prototype bounded reusable staging or an atomically published shared-memory descriptor ring
  where supported; retain an explicit transferable-buffer path.
- [ ] Define producer/consumer ownership, generation, slot capacity, wraparound, detach behavior and
  backpressure before publishing bytes.
- [ ] Exercise reset, teardown, ring-full, slow consumer and serial/threaded modes with adversarial
  scheduling.
- [ ] Compare both variants under the same real API workloads and retain a variant only after
  reduced bytes/CPU time translates into accepted end-to-end gains.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run `node --test web/js/vm-worker/gpu-3d.test.mjs` and `make web-pkg`; both serial and threaded
  builds must execute the transport validation workload.
- Tests prove no partial publication, stale slot reuse, use-after-detach, unbounded growth or missed
  wakeup.
- Report copies/bytes saved and complete-frame/input results; shared memory is never described as
  GPU zero-copy unless directly demonstrated.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
