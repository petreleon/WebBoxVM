# G06 — Replace fixed shader-shape dispatch with compiled VirGL shader programs

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G06
Depends: G02, S02, S03, S04, S05, S06, S07, S12
Evidence: pending

Prerequisite lists: [G02](../../01-mesa-ingress/02-object-state-lifecycle/README.md), [S02](../../../02-shaders/01-frontends/02-tgsi-parser/README.md), [S03](../../../02-shaders/01-frontends/03-tgsi-arithmetic/README.md), [S04](../../../02-shaders/01-frontends/04-tgsi-control/README.md), [S05](../../../02-shaders/02-backend/01-wgsl-emission/README.md), [S06](../../../02-shaders/02-backend/02-stage-interfaces/README.md), [S07](../../../02-shaders/02-backend/03-texture-operations/README.md), [S12](../../../02-shaders/04-validation/02-cache/README.md).

## Outcome

Mesa shader-object uploads compile through typed IR and WGSL with correct stage linkage and resource
bindings.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/shader.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/shader.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/context/shader/chunks.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/shader/chunks.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/context/draw.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/draw.rs)
- [web/js/webgpu-3d.js](../../../../../web/js/webgpu-3d.js)

## Checklist

- [ ] Replace the finite ShaderProgram shape enum at the VirGL boundary with validated program
  identities, reflection, and compiled-pipeline metadata.
- [ ] Preserve chunk ordering, source/token bounds, stage identity, uniforms, varyings, and
  compile/link diagnostics across the Rust/browser boundary.
- [ ] Reject unsupported operations with explicit compilation errors and preserve the previous valid
  program on failed replacement; prohibit shader-text fixture whitelists.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- At least three generated shader variants outside the old fixed shapes render correctly, including
  control flow and independently varied uniform/texture inputs.
- Run cargo test -p emulator --lib virgl_shader --quiet and cargo test -p emulator --lib
  virgl_matrix --quiet; also record real GPU compilation errors and pixel comparisons from the
  shader harness.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
