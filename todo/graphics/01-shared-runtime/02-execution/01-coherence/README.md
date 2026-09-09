# R05 — Define CPU/GPU resource ownership transitions

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: R05
Depends: R03, R04
Evidence: pending

Prerequisite lists: [R03](../../01-resources/03-buffer-transfers/README.md), [R04](../../01-resources/04-texture-layout/README.md).

## Outcome

Resident GPU results and guest-visible memory remain coherent without unconditional CPU
rerasterization.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/residency.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/residency.rs)
- [emulator/src/devices/virtio_gpu/completion/readback.rs](../../../../../emulator/src/devices/virtio_gpu/completion/readback.rs)
- [web/js/webgpu-readback.js](../../../../../web/js/webgpu-readback.js)

## Checklist

- [ ] Model CPU-valid, GPU-valid, synchronized and pending versions per buffer range or texture
  subresource.
- [ ] Schedule upload on GPU consumption and readback on guest observation; retain current
  authoritative behavior until equivalence is proven.
- [ ] Test CPU-write/GPU-read, GPU-write/CPU-read, copy/readback races and invalidation after reset.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Reference bytes match through alternating CPU and GPU writers and overlapping subranges.
- A guest read cannot complete before its required GPU result is materialized.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
