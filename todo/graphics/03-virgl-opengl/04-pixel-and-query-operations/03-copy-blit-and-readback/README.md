# G12 — Implement VirGL copy/blit and explicit readback synchronization

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G12
Depends: G05, G08, R05, R06
Evidence: pending

Prerequisite lists: [G05](../../02-resource-and-shader-bindings/02-texture-subresource-bindings/README.md), [G08](../../03-draw-state/02-framebuffer-depth-stencil/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md).

## Outcome

Copy, blit, and guest readback obey subresource bounds, conversion rules, and producer completion.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/copy.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/copy.rs)
- [emulator/src/devices/virtio_gpu/three_d/residency/copy.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/residency/copy.rs)
- [emulator/src/devices/virtio_gpu/resource_transfer/readback.rs](../../../../../emulator/src/devices/virtio_gpu/resource_transfer/readback.rs)
- [web/js/webgpu-readback.js](../../../../../web/js/webgpu-readback.js)

## Checklist

- [ ] Route VirGL copies and readbacks through shared subresource/ownership operations instead of
  full-target-only private envelopes.
- [ ] Implement each mandatory blit filter, format conversion, overlap, and depth/stencil rule as an
  independently verified descendant.
- [ ] Wait only for dependencies needed by the requested readback and report invalid or lost
  producers without exposing stale or uninitialized guest bytes.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Reference checks cover partial rectangles, nonzero levels/layers, row stride, flip direction, and
  source mutation after queued copy; allowed overlaps follow the pinned specification.
- Run cargo test -p emulator --lib virgl_copy --quiet and cargo test -p emulator --lib
  virgl_readback --quiet; capture bytes transferred and confirm no readback for pure resident
  copies.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
