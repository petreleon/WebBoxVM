# V13 — Execute Vulkan graphics draw commands through WebGPU

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V13
Depends: V10, V11, V12, R03
Evidence: pending

Prerequisite lists: [V10](../../04-pipeline-and-execution/01-descriptors-and-push-constants/README.md), [V11](../../04-pipeline-and-execution/02-shader-and-pipeline-creation/README.md), [V12](../../04-pipeline-and-execution/03-render-pass-attachments/README.md), [R03](../../../01-shared-runtime/01-resources/03-buffer-transfers/README.md).

## Outcome

Recorded Vulkan draws use guest vertex/index resources, dynamic state, and pipeline bindings to
produce real browser GPU output.

## Starting points

- [web/js/webgpu-3d.js](../../../../../web/js/webgpu-3d.js)
- [web/js/webgpu-virgl-vertex-cache.js](../../../../../web/js/webgpu-virgl-vertex-cache.js)
- [emulator/src/devices/virtio_gpu/three_d/virgl/draw/vertices.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/draw/vertices.rs)

## Checklist

- [ ] Implement direct indexed/nonindexed draw execution with instance/base parameters and
  profile-required dynamic state.
- [ ] Add independently checked descendants for indirect draws, restart, raster/blend/depth/stencil
  variants, and any required advanced graphics stage.
- [ ] Reject invalid encoded records and unsupported feature use before execution; preserve draw
  order and do not repack every vertex through fixed CPU triangle snapshots.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A stock Mesa Venus guest renders multiple frames with changing buffers, indexed instances,
  clipping, and overlapping depth/blend tests, compared with the pinned Vulkan reference.
- Browser instrumentation proves actual GPU draw execution and resource reuse; every advertised draw
  mode/state has a deterministic named test.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
