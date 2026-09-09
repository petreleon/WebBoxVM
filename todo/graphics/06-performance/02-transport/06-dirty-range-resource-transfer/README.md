# P06 — Experiment with generation-aware dirty resource transfers

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P06
Depends: P03, R02, R03, R04, R05
Evidence: pending

Prerequisite lists: [P03](../../01-measurement/03-matched-baseline-runs/README.md), [R02](../../../01-shared-runtime/01-resources/02-object-identity/README.md), [R03](../../../01-shared-runtime/01-resources/03-buffer-transfers/README.md), [R04](../../../01-shared-runtime/01-resources/04-texture-layout/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md).

## Outcome

Buffer and texture updates transfer only required changed regions with exact coherence and bounded
metadata.

## Starting points

- [emulator/src/devices/virtio_gpu/resource_transfer.rs](../../../../../emulator/src/devices/virtio_gpu/resource_transfer.rs)
- [emulator/src/devices/virtio_gpu/resource_transfer/readback.rs](../../../../../emulator/src/devices/virtio_gpu/resource_transfer/readback.rs)
- [web/js/webgpu-virgl-texture-cache.js](../../../../../web/js/webgpu-virgl-texture-cache.js)
- [web/js/webgpu-virgl-vertex-cache.js](../../../../../web/js/webgpu-virgl-vertex-cache.js)

## Checklist

- [ ] Measure byte scanning, hash/equality checking, full snapshots and uploads for unchanged/static
  and partially changing real API resources.
- [ ] Track validated resource generation plus dirty ranges/subresources instead of rescanning every
  immutable resource snapshot; coalesce bounded adjacent ranges.
- [ ] Preserve guest CPU writes, host GPU writes, map/unmap, stride/format conversion, partial
  readbacks and resource ID reuse semantics.
- [ ] Add differential full-upload versus dirty-upload cases including overlapping updates, boundary
  rows and generation reuse.
- [ ] Measure copied/uploaded bytes, CPU scanning and native ratios; retain full-copy fallback for
  cases whose exact dirty ownership cannot be established.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run `node --test web/js/webgpu-virgl-texture-cache.test.mjs
  web/js/webgpu-virgl-vertex-cache.test.mjs web/js/gpu-virgl-resident-readback.test.mjs` and
  relevant new Rust coherence tests.
- Reference/full-transfer and optimized outputs match at P01 tolerances after partial writes and
  readbacks; invalidation never produces stale pixels.
- Savings persist on real guest API workloads and memory metadata remains within frozen limits.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
