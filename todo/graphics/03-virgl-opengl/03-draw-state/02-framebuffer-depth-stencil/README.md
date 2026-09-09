# G08 — Implement VirGL framebuffer attachment and depth/stencil state

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G08
Depends: G05, G06, R05
Evidence: pending

Prerequisite lists: [G05](../../02-resource-and-shader-bindings/02-texture-subresource-bindings/README.md), [G06](../../02-resource-and-shader-bindings/03-general-shader-objects/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md).

## Outcome

Framebuffer attachments and depth/stencil behavior are validated against shared subresource identity
and backend support.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/context/framebuffer.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/framebuffer.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/context/depth.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/depth.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/depth.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/depth.rs)
- [web/js/webgpu-virgl-output-target.js](../../../../../web/js/webgpu-virgl-output-target.js)

## Checklist

- [ ] Map framebuffer color/depth/stencil attachment ranges and load/store behavior to explicit
  render-target descriptors.
- [ ] Implement required stencil faces/operations/masks, depth range/write/compare, and attachment
  completeness with separately verifiable descendants for MRT or MSAA requirements.
- [ ] Reject incompatible sample counts/formats and incomplete targets before rendering; preserve
  unaffected attachments and masked channels on clears.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Guest FBO tests compare depth rejection, front/back stencil, masked clears, and attachment
  switches against the pinned reference renderer.
- Run cargo test -p emulator --lib virgl_depth --quiet; required MRT/MSAA/resolve children must have
  independent evidence before their capabilities become visible.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
