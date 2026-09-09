# S07 — Lower general shader texture operations

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S07
Depends: S05, S06, R04
Evidence: pending

Prerequisite lists: [S05](../01-wgsl-emission/README.md), [S06](../02-stage-interfaces/README.md), [R04](../../../01-shared-runtime/01-resources/04-texture-layout/README.md).

## Outcome

Sampling behavior follows the resource and sampler state rather than fixed texture templates.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/context/sampler.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/sampler.rs)
- [web/js/webgpu-virgl-texture.js](../../../../../web/js/webgpu-virgl-texture.js)

## Checklist

- [ ] Split sample, explicit LOD/gradients, texel fetch/gather and shadow-compare families into
  child tasks.
- [ ] Implement dimensionality, arrays, integer/depth samples and swizzles according to the feature
  matrix.
- [ ] Validate binding/type compatibility and record derivative and filtering restrictions.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Mip/array/shadow reference images and exact integer fetch outputs match the native renderer.
- Bad sampler/resource pairs fail before dispatch and leave prior state usable.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
