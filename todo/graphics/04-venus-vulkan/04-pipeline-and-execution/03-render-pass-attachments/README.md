# V12 — Implement Vulkan render-pass attachment and subpass behavior

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V12
Depends: V09, V11, R04
Evidence: pending

Prerequisite lists: [V09](../../03-command-and-sync-state/03-timeline-and-memory-dependencies/README.md), [V11](../02-shader-and-pipeline-creation/README.md), [R04](../../../01-shared-runtime/01-resources/04-texture-layout/README.md).

## Outcome

Render passes preserve attachment load/store, clear, resolve, and required subpass dependencies over
shared images.

## Starting points

- [web/js/webgpu-virgl-output-target.js](../../../../../web/js/webgpu-virgl-output-target.js)
- [web/js/webgpu-virgl-color-target.js](../../../../../web/js/webgpu-virgl-color-target.js)
- [emulator/src/devices/virtio_gpu/three_d/virgl/context/framebuffer.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/framebuffer.rs)

## Checklist

- [ ] Implement render-pass/framebuffer lifecycle and compatibility checks from explicit image
  subresources and pipeline declarations.
- [ ] Translate attachment load/store and clear operations exactly; split multiple subpasses, input
  attachments, multisample resolves, and dynamic rendering into child tasks when the profile
  requires them.
- [ ] Reject invalid attachments or unsupported dependency patterns safely and preserve attachment
  contents across any backend pass splits.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A Vulkan guest fixture renders, reloads, and reads back attachments across passes with nontrivial
  clear/store/discard behavior and reference-checked pixels.
- Every mandatory subpass/resolve/dynamic-rendering case has its own evidence before profile
  exposure; backend pass splitting cannot alter pixel or synchronization results.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
