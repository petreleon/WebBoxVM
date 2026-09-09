# S10 — Implement shader storage, compute and memory semantics

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S10
Depends: S09, R05
Evidence: pending

Prerequisite lists: [S09](../02-spirv-lowering/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md).

## Outcome

Advertised compute/storage operations preserve visibility and synchronization semantics.

## Starting points

- [research/webgpu-acceleration.md](../../../../../research/webgpu-acceleration.md)
- [web/js/webgpu-session.js](../../../../../web/js/webgpu-session.js)
- [emulator/src/devices/virtio_gpu/three_d/pending.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/pending.rs)

## Checklist

- [ ] Create separate child tasks for SSBO/storage-image access, atomics, workgroup memory and
  barriers.
- [ ] Map exact semantics to WebGPU or implement measured emulation for mandatory unsupported
  operations.
- [ ] Test workgroup limits, bounds, race-free memory litmus programs and dispatch completion.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Deterministic compute outputs match native Vulkan/GL references for each enabled feature.
- A browser memory-model mismatch keeps the related advertised feature disabled and mandatory
  profile blocked.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
