# P09 — Experiment with safe pass coalescing and state reuse

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P09
Depends: P03, R05, R06, S12
Evidence: pending

Prerequisite lists: [P03](../../01-measurement/03-matched-baseline-runs/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md), [S12](../../../02-shaders/04-validation/02-cache/README.md).

## Outcome

Compatible work uses fewer render passes and redundant bindings while preserving API-visible order.

## Starting points

- [web/js/webgpu-virgl-solid-batch.js](../../../../../web/js/webgpu-virgl-solid-batch.js)
- [web/js/webgpu-virgl-material-batch.js](../../../../../web/js/webgpu-virgl-material-batch.js)
- [web/js/webgpu-virgl-texture-cache.js](../../../../../web/js/webgpu-virgl-texture-cache.js)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/batch.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/batch.rs)

## Checklist

- [ ] Measure render pass, pipeline, bind group and attachment setup counts on draw-heavy real API
  frames.
- [ ] Derive compatibility keys from attachments, load/store operations, formats, sample count and
  dependencies; merge only adjacent legally compatible work.
- [ ] Elide unchanged bindings using exact validated state identity and include device generation in
  cached GPU object ownership.
- [ ] Add alpha/depth/stencil ordering, query, readback, barrier and attachment-change cases that
  must split passes.
- [ ] Compare pass/bind counts, compilation stalls and complete-frame performance against disabled
  coalescing.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run `node --test web/js/gpu-virgl-solid-batch.test.mjs web/js/gpu-virgl-material-batch.test.mjs
  web/js/gpu-virgl-depth-batch.test.mjs`.
- Differential outputs and API query/fence results match; incompatible order-sensitive operations
  remain separated.
- Retain coalescing only when P01 workload measurements improve and shader/cache memory budgets
  remain bounded.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
