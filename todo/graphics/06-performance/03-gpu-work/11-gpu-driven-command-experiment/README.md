# P11 — Test GPU-driven indirect work where API semantics allow it

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P11
Depends: P03, R03, R04, R05, R06, S12, I04
Evidence: pending

Prerequisite lists: [P03](../../01-measurement/03-matched-baseline-runs/README.md), [R03](../../../01-shared-runtime/01-resources/03-buffer-transfers/README.md), [R04](../../../01-shared-runtime/01-resources/04-texture-layout/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md), [S12](../../../02-shaders/04-validation/02-cache/README.md), [I04](../../../05-guest-validation/02-independent-validation/01-conformance-runner/README.md).

## Outcome

An isolated capability-gated experiment evaluates indirect drawing or GPU culling without
advertising unsupported API features.

## Starting points

- [web/js/webgpu-3d.js](../../../../../web/js/webgpu-3d.js)
- [web/js/webgpu-3d-resources.js](../../../../../web/js/webgpu-3d-resources.js)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/draw.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/draw.rs)
- [research/webgpu-acceleration.md](../../../../../research/webgpu-acceleration.md)

## Checklist

- [ ] Select one measured CPU submission bottleneck whose required indirect/compute semantics fit
  the frozen WebGPU/API profile.
- [ ] Specify exact indirect buffer layout, draw count bounds, index/instance semantics,
  synchronization and fallback behavior.
- [ ] Prototype GPU culling or indirect command generation for that declared lane with legal
  barriers and a deterministic direct-draw reference.
- [ ] Exercise zero/max draw counts, invalid ranges, aliasing, device loss and CPU readback of
  relevant generated data.
- [ ] Compare correct completed-work throughput including generation cost; reject the approach if
  unsupported semantics or overhead erase its gain.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- The relevant I04 conformance shard and direct/indirect differential cases pass before any
  capability is exposed.
- GPU work neither invents invisible draw outcomes nor bypasses bounds, queries, barriers or
  guest-visible memory writes.
- Record the accepted feature/profile boundary and full P01 A/B result; a failed experiment may
  close as rejected but cannot count as compatibility or near-native success.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
