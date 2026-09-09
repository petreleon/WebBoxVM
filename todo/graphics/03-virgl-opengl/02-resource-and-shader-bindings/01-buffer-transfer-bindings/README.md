# G04 — Wire VirGL buffer ranges to shared transfer and binding semantics

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G04
Depends: G02, R03
Evidence: pending

Prerequisite lists: [G02](../../01-mesa-ingress/02-object-state-lifecycle/README.md), [R03](../../../01-shared-runtime/01-resources/03-buffer-transfers/README.md).

## Outcome

Mesa vertex, index, and uniform buffer updates preserve byte ranges, offsets, and later draw
visibility.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/resource.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/resource.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/inline.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/inline.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/uniform.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/uniform.rs)
- [emulator/src/devices/virtio_gpu/tests/virgl_buffer.rs](../../../../../emulator/src/devices/virtio_gpu/tests/virgl_buffer.rs)

## Checklist

- [ ] Map VirGL buffer creation, inline writes, transfer offsets, and supported bind flags into
  shared buffer storage without fixed demo-size assumptions.
- [ ] Implement the profile's vertex/index/uniform range bindings with explicit alignment,
  zero-size, and lifetime rules.
- [ ] Reject overflow, detached backing, invalid bind combinations, and out-of-range access
  transactionally; give any additional buffer class its own child task.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A stock Mesa buffer-update probe renders from nonzero vertex/index/uniform offsets and reads back
  the independently expected pixels after a second update.
- Run cargo test -p emulator --lib virgl_buffer --quiet and cargo test -p emulator --lib
  virgl_uniform --quiet; record named boundary cases alongside the guest evidence.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
