# V14 — Execute Vulkan compute dispatch and storage effects

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V14
Depends: V09, V10, V11, S10
Evidence: pending

Prerequisite lists: [V09](../../03-command-and-sync-state/03-timeline-and-memory-dependencies/README.md), [V10](../../04-pipeline-and-execution/01-descriptors-and-push-constants/README.md), [V11](../../04-pipeline-and-execution/02-shader-and-pipeline-creation/README.md), [S10](../../../02-shaders/03-spirv/03-storage-compute/README.md).

## Outcome

Guest compute dispatch produces correctly synchronized storage-buffer/image effects through compiled
WebGPU compute pipelines.

## Starting points

- [web/js/webgpu-3d.js](../../../../../web/js/webgpu-3d.js)
- [web/js/webgpu-3d-resources.js](../../../../../web/js/webgpu-3d-resources.js)
- [emulator/src/devices/virtio_gpu/resource_transfer/readback.rs](../../../../../emulator/src/devices/virtio_gpu/resource_transfer/readback.rs)

## Checklist

- [ ] Implement dispatch dimensions, workgroup limits, and storage bindings with the shader
  backend's declared memory model.
- [ ] Exercise shared memory, barriers, atomics, and storage-image behavior required by the profile
  as separately verified descendants.
- [ ] Reject unsupported subgroup/capability combinations and invalid dimensions without running a
  CPU replacement under an accelerated result label.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Guest vector arithmetic, workgroup reduction, and atomic contention probes produce independently
  computed exact integer outputs or documented float tolerances.
- A compute-to-graphics or compute-to-transfer dependency test passes without host roundtrip
  synchronization; every advertised shader memory capability has a negative and positive case.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
