# V09 — Implement required timeline values and Vulkan memory dependencies

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V09
Depends: V08, R05, R07, F03
Evidence: pending

Prerequisite lists: [V08](../02-queue-fences-binary-semaphores/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md), [R07](../../../01-shared-runtime/02-execution/03-scheduler/README.md), [F03](../../../00-foundation/01-contract/03-feature-matrix/README.md).

## Outcome

Timeline operations and pipeline memory dependencies make resource writes visible to the correct
later consumers.

## Starting points

- [emulator/src/devices/virtio_gpu/fence.rs](../../../../../emulator/src/devices/virtio_gpu/fence.rs)
- [emulator/src/devices/virtio_gpu/three_d/residency.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/residency.rs)
- [web/js/webgpu-3d-resources.js](../../../../../web/js/webgpu-3d-resources.js)

## Checklist

- [ ] Implement required timeline semaphore monotonic values, host query/signal/wait, and queued
  waits/signals; keep optional timeline support hidden until complete.
- [ ] Translate buffer/image stage-access barriers, image layouts, and queue ownership into the
  shared hazard model with precise subresource ranges.
- [ ] Split events, cross-queue transfers, and other mandatory dependency mechanisms into child
  tasks; reject unsupported required semantics rather than equating every barrier with a global
  stall.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Guest tests cover transfer-to-vertex, transfer-to-sampling, compute-write-to-readback, image
  layout reuse, and timeline values signaled before/after waits.
- Results match an independent Vulkan implementation; a report names every supported queue
  family/stage/access/layout and every still-blocked mandatory dependency.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
