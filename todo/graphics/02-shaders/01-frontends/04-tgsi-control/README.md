# S04 — Lower TGSI control flow and indirect addressing

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S04
Depends: S03, S05
Evidence: pending

Prerequisite lists: [S03](../03-tgsi-arithmetic/README.md), [S05](../../02-backend/01-wgsl-emission/README.md).

## Outcome

Ordinary branch/loop shaders no longer depend on fixed straight-line patterns.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse/vertex.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse/vertex.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse/fragment.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse/fragment.rs)

## Checklist

- [ ] Lower conditional blocks, loops, breaks and indirect register/constant access into validated
  control flow.
- [ ] Preserve derivative uniformity rules and reject unsupported divergent cases accurately.
- [ ] Bound compile complexity and add deeply nested/invalid control-flow regression fixtures.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Standalone browser shader replay of loop/branch-heavy programs matches the pinned reference across
  varying inputs; no completed guest driver is required here.
- Malformed programs and excessive compilation work fail with a diagnostic without blocking the
  browser.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
