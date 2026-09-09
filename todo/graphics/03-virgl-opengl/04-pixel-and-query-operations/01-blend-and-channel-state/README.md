# G10 — Implement profile blend equations and channel masks

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G10
Depends: G08, R01
Evidence: pending

Prerequisite lists: [G08](../../03-draw-state/02-framebuffer-depth-stencil/README.md), [R01](../../../01-shared-runtime/01-resources/01-capability-registry/README.md).

## Outcome

VirGL blend objects map supported independent equations, factors, constants, and masks to validated
render pipelines.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/context/blend.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/blend.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/blend.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/blend.rs)
- [emulator/src/devices/virtio_gpu/tests/virgl_blend_state.rs](../../../../../emulator/src/devices/virtio_gpu/tests/virgl_blend_state.rs)

## Checklist

- [ ] Generate blend-state test cases from the declared profile and map separate color/alpha
  equations, factors, constants, and masks.
- [ ] Implement unsupported backend cases only through proven semantic lowering and separate child
  tasks, including logic operations or independent MRT blends when required.
- [ ] Reject unsupported combinations before GPU execution and ensure failed pipeline creation
  cannot acknowledge a successful guest draw.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A data-driven table compares every advertised blend equation/factor and RGBA mask on nontrivial
  source/destination values against the reference renderer.
- Run cargo test -p emulator --lib virgl_blend_state --quiet and cargo test -p emulator --lib
  virgl_solid_batch --quiet; retain ordered source-over regression results.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
