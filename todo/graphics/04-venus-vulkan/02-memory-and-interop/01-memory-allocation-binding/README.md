# V04 — Implement Venus memory allocation and resource binding

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V04
Depends: V02, R03, R04, R05
Evidence: pending

Prerequisite lists: [V02](../../01-real-protocol/02-renderer-object-lifetimes/README.md), [R03](../../../01-shared-runtime/01-resources/03-buffer-transfers/README.md), [R04](../../../01-shared-runtime/01-resources/04-texture-layout/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md).

## Outcome

Vulkan allocation requirements, memory types, blob IDs, and resource binds have a consistent
renderer-backed ownership model.

## Starting points

- [emulator/src/devices/virtio_gpu/blob/create.rs](../../../../../emulator/src/devices/virtio_gpu/blob/create.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/blob.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/blob.rs)
- [emulator/src/devices/virtio_gpu/resource.rs](../../../../../emulator/src/devices/virtio_gpu/resource.rs)
- [research/renderer-blob-ordering.md](../../../../../research/renderer-blob-ordering.md)

## Checklist

- [ ] Implement real renderer allocation records before matching nonzero blob creation, replacing
  the private WBL1 dependency for Venus contexts.
- [ ] Return truthful size/alignment/memoryTypeBits and enforce allocation budgets, binding offsets,
  allowed aliasing, and allocation/resource lifetime rules.
- [ ] Reject incompatible memory types, overflow, misalignment, duplicate/invalid binds, and failed
  allocations without consuming the pending renderer object.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Stock Mesa allocation/bind traces exercise at least one buffer and one image, nonzero aligned
  offsets, and reproducible OOM with exact required failure cleanup.
- Run cargo test -p emulator --lib blob --quiet; a new Venus allocation fixture must prove real
  allocation opcodes rather than merely preserving the private blob probe.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
