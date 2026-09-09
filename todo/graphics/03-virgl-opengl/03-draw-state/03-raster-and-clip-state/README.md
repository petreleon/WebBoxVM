# G09 — Implement profile rasterization and clipping state

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G09
Depends: G07, G08, S06
Evidence: pending

Prerequisite lists: [G07](../01-vertex-and-index-fetch/README.md), [G08](../02-framebuffer-depth-stencil/README.md), [S06](../../../02-shaders/02-backend/02-stage-interfaces/README.md).

## Outcome

Viewport/scissor, culling, winding, depth bias, and required primitive rasterization match the guest
API coordinate conventions.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/context/rasterizer.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/rasterizer.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/state.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/state.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/draw/transform.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/draw/transform.rs)

## Checklist

- [ ] Map rasterizer fields to a canonical backend descriptor with explicit clip/depth conventions
  and viewport/scissor origins.
- [ ] Add separate fixtures for each mandatory raster state; create dedicated descendants for
  line/point/clip-distance behavior where WebGPU needs emulation.
- [ ] Return a defined unsupported-state error until a required lowering exists rather than silently
  forcing the current unit-line/no-cull state.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Reference images include all viewport quadrants, reversed winding, near-plane clipping, scissor
  edges, and enabled/disabled depth bias with stated edge tolerances.
- Run cargo test -p emulator --lib virgl_viewport_state --quiet and cargo test -p emulator --lib
  virgl_draw --quiet; GPU and guest-readback paths agree.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
