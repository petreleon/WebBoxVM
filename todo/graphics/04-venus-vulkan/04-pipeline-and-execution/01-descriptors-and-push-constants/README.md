# V10 — Implement descriptor-set and push-constant binding state

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V10
Depends: V04, V07, S05
Evidence: pending

Prerequisite lists: [V04](../../02-memory-and-interop/01-memory-allocation-binding/README.md), [V07](../../03-command-and-sync-state/01-command-buffer-lifecycle/README.md), [S05](../../../02-shaders/02-backend/01-wgsl-emission/README.md).

## Outcome

Vulkan shader resources resolve through validated descriptor layouts, updates, ranges, dynamic
offsets, and push constants.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/context/draw.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/draw.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/uniform.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/uniform.rs)
- [web/js/webgpu-3d-resources.js](../../../../../web/js/webgpu-3d-resources.js)

## Checklist

- [ ] Implement descriptor layout/pool/set lifecycle and supported buffer/image/sampler updates with
  shader reflection and shared resource identities.
- [ ] Map dynamic offsets and push-constant byte ranges/stage visibility to stable backend bindings;
  cover descriptor arrays and update lifetime rules in dedicated children when required.
- [ ] Reject invalid protocol handles and range/limit violations before encoding; prevent descriptor
  mutation from accidentally changing already recorded semantics.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A guest program rebinds sets and nonzero dynamic offsets, varies push constants between draws, and
  verifies independent image/buffer outputs against a reference.
- Tests cover pool reset/destruction, stale resources, partial writes/copies, bounds, and all
  descriptor types advertised by the target profile.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
