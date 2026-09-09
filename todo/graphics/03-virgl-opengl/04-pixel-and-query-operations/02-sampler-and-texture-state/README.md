# G11 — Implement profile sampler state beyond the three fixed words

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G11
Depends: G05, G06, S07
Evidence: pending

Prerequisite lists: [G05](../../02-resource-and-shader-bindings/02-texture-subresource-bindings/README.md), [G06](../../02-resource-and-shader-bindings/03-general-shader-objects/README.md), [S07](../../../02-shaders/02-backend/03-texture-operations/README.md).

## Outcome

VirGL sampler objects express supported address modes, filters, LOD, comparison, and border behavior
without magic-word matching.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/context/sampler.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/sampler.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/sampler.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/sampler.rs)
- [web/js/webgpu-virgl-texture-cache.js](../../../../../web/js/webgpu-virgl-texture-cache.js)

## Checklist

- [ ] Decode sampler fields individually into a canonical descriptor and key caches by the full
  effective state.
- [ ] Implement each mandatory address/filter/LOD/comparison case with explicit shader lowering
  where the browser lacks a direct equivalent.
- [ ] Reject nonfinite LOD values and unsupported combinations without substituting clamp-nearest;
  split anisotropy/border/comparison requirements into descendants when necessary.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Guest sampling fixtures distinguish minification/magnification, mip transitions,
  negative/out-of-range coordinates, compare sampling, and any exposed border behavior.
- Run cargo test -p emulator --lib virgl_sampler --quiet; repeated binds and state mutations must
  not reuse an incompatible cached sampler.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
