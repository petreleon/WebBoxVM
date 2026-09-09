# V11 — Implement Vulkan shader module and pipeline creation

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V11
Depends: V10, S08, S09, S05, S06, S07, S12
Evidence: pending

Prerequisite lists: [V10](../01-descriptors-and-push-constants/README.md), [S08](../../../02-shaders/03-spirv/01-spirv-validation/README.md), [S09](../../../02-shaders/03-spirv/02-spirv-lowering/README.md), [S05](../../../02-shaders/02-backend/01-wgsl-emission/README.md), [S06](../../../02-shaders/02-backend/02-stage-interfaces/README.md), [S07](../../../02-shaders/02-backend/03-texture-operations/README.md), [S12](../../../02-shaders/04-validation/02-cache/README.md).

## Outcome

Guest SPIR-V modules create validated backend pipelines with accurate specialization, layout, and
failure behavior.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/shader.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/shader.rs)
- [web/js/webgpu-3d.js](../../../../../web/js/webgpu-3d.js)
- [web/js/webgpu-errors.js](../../../../../web/js/webgpu-errors.js)

## Checklist

- [ ] Connect validated SPIR-V translation to Venus shader modules, entry-point selection,
  specialization constants, and pipeline-layout compatibility.
- [ ] Implement graphics/compute pipeline creation records as separate small descendants with
  immutable effective state and cache keys covering every relevant input.
- [ ] Propagate validation/compiler/device failures to the correct guest result and diagnostics;
  never substitute a canned shader after translation failure.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A guest pipeline probe compiles multiple non-template SPIR-V modules and independently varies
  specialization constants, entry points, and descriptor layouts with correct output.
- Malformed module/capability cases return the documented errors without leaked pipeline handles;
  captured WGSL and real WebGPU compilation results accompany pixel/buffer oracle comparisons.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
