# R06 — Implement ordered completion and guest fences

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: R06
Depends: R02, R05
Evidence: pending

Prerequisite lists: [R02](../../01-resources/02-object-identity/README.md), [R05](../01-coherence/README.md).

## Outcome

Guest completion signals correspond to completed effects with correct ordering and errors.

## Starting points

- [emulator/src/devices/virtio_gpu/fence.rs](../../../../../emulator/src/devices/virtio_gpu/fence.rs)
- [emulator/src/devices/virtio_gpu/three_d/pending.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/pending.rs)
- [emulator/src/devices/virtio_gpu/queue.rs](../../../../../emulator/src/devices/virtio_gpu/queue.rs)
- [web/js/gpu-display.js](../../../../../web/js/gpu-display.js)

## Checklist

- [ ] Define submission, resource visibility, guest fence and browser presentation as distinct
  milestones.
- [ ] Implement per-context/timeline completion bookkeeping and error propagation; preserve
  descriptor/IRQ ordering.
- [ ] Test delayed, reordered, duplicate and lost acknowledgments, including resource destruction
  during pending work.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run cargo test -p emulator --lib fence --quiet and require nonzero tests.
- A forced GPU failure never becomes a success fence, and no fence signals before dependent memory
  visibility.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
