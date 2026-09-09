# P08 — Extend measured GPU residency across render dependencies

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P08
Depends: P03, R02, R04, R05, R08
Evidence: pending

Prerequisite lists: [P03](../../01-measurement/03-matched-baseline-runs/README.md), [R02](../../../01-shared-runtime/01-resources/02-object-identity/README.md), [R04](../../../01-shared-runtime/01-resources/04-texture-layout/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md), [R08](../../../01-shared-runtime/02-execution/04-device-loss/README.md).

## Outcome

Eligible real API render/sample/copy chains retain GPU ownership and avoid unnecessary
upload/readback cycles.

## Starting points

- [research/virgl-resource-residency.md](../../../../../research/virgl-resource-residency.md)
- [emulator/src/devices/virtio_gpu/completion/resident.rs](../../../../../emulator/src/devices/virtio_gpu/completion/resident.rs)
- [web/js/webgpu-virgl-output-target.js](../../../../../web/js/webgpu-virgl-output-target.js)
- [web/js/webgpu-virgl-resident-copy.js](../../../../../web/js/webgpu-virgl-resident-copy.js)

## Checklist

- [ ] Trace real API frame dependencies and identify render targets redundantly copied through CPU
  shadows.
- [ ] Extend residency by resource/subresource identity and producer version; represent render,
  sample, copy and CPU-read dependency transitions explicitly.
- [ ] Retain bounded memory accounting and retirement after GPU work; reconstruct or
  deterministically invalidate ownership on loss/reset.
- [ ] Verify render-to-texture, partial update/readback, overlapping copy, multi-pass use and
  destroyed/reused identities against a synchronized reference.
- [ ] Measure eliminated maps/uploads, peak resident bytes and complete-frame gain; record exact
  unsupported residency cases.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run `node --test web/js/webgpu-virgl-output-target.test.mjs
  web/js/gpu-virgl-resident-copy.test.mjs web/js/gpu-virgl-resident-sample.test.mjs
  web/js/gpu-virgl-resident-readback.test.mjs`.
- GPU and guest CPU observations match after explicit synchronization; delayed completion never
  publishes a retired producer.
- The selected real API chain has fewer round trips and passes P01 memory/performance budgets
  without silently skipping CPU-visible effects.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
