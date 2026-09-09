# G05 — Wire VirGL texture creation and views to shared subresources

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G05
Depends: G02, R04
Evidence: pending

Prerequisite lists: [G02](../../01-mesa-ingress/02-object-state-lifecycle/README.md), [R04](../../../01-shared-runtime/01-resources/04-texture-layout/README.md).

## Outcome

Texture dimensions, mip/layer ranges, formats, and view swizzles are resolved through shared
resource layouts.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/resource.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/resource.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/context/sampler.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/sampler.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/sampler.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/sampler.rs)

## Checklist

- [ ] Replace level-zero and fixed-small-texture assumptions with validated shared subresource
  descriptors for the formats and targets in the profile.
- [ ] Implement VirGL surface/sampler-view range and swizzle decoding; split cube, array, 3D,
  compressed, and multisample target requirements into separate descendants when required.
- [ ] Reject incompatible formats, missing levels, illegal layer ranges, feedback hazards, and stale
  views before emitting GPU work.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A Mesa texture fixture samples distinct mip/layer/swizzle patterns and compares readback to an
  independent renderer with a declared tolerance.
- Run cargo test -p emulator --lib virgl_transfer --quiet and cargo test -p emulator --lib
  virgl_sampler --quiet; each advertised target/format must have an exercised child case.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
