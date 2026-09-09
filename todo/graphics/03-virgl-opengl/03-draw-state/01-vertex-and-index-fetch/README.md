# G07 — Implement profile vertex/index fetching and draw parameters

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G07
Depends: G04, G06, S06
Evidence: pending

Prerequisite lists: [G04](../../02-resource-and-shader-bindings/01-buffer-transfer-bindings/README.md), [G06](../../02-resource-and-shader-bindings/03-general-shader-objects/README.md), [S06](../../../02-shaders/02-backend/02-stage-interfaces/README.md).

## Outcome

VirGL draws consume validated layouts, indices, offsets, instance divisors, and base parameters
without CPU vertex-shape normalization.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/draw/vertices.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/draw/vertices.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/context/vertex/layout.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/vertex/layout.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/draw.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/draw.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/draw/primitive.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/draw/primitive.rs)

## Checklist

- [ ] Implement the declared profile's vertex format conversion and indexed/nonindexed parameter
  mapping through resident buffers.
- [ ] Cover base vertex, instance count/divisor, primitive restart, and required primitive modes as
  separately checked child cases if their combined scope is too large.
- [ ] Reject invalid ranges and overflow before submission while preserving clipping in the GPU
  pipeline instead of rejecting valid off-screen vertices.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Independent reference images cover interleaved/split attributes, u16/u32 indices, nonzero bases,
  multiple instances, restart boundaries, and triangles crossing the clip volume.
- Run cargo test -p emulator --lib virgl_indexed_draw --quiet and cargo test -p emulator --lib
  virgl_triangle_primitives --quiet; legacy bounded draw fixtures remain green.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
