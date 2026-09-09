# V07 — Implement Vulkan command pool and buffer lifecycle

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V07
Depends: V02, R07
Evidence: pending

Prerequisite lists: [V02](../../01-real-protocol/02-renderer-object-lifetimes/README.md), [R07](../../../01-shared-runtime/02-execution/03-scheduler/README.md).

## Outcome

Command buffers preserve recording, executable, pending, reset, and reuse semantics across Venus
submissions.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/pending.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/pending.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream.rs)
- [web/js/vm-worker/gpu-3d.js](../../../../../web/js/vm-worker/gpu-3d.js)

## Checklist

- [ ] Implement command pool/buffer allocation, begin/end/reset/free transitions and retained
  references using a backend-neutral recorded command sequence.
- [ ] Cover primary/secondary buffers, inheritance, simultaneous-use, and one-time-submit
  requirements in individually tested descendants where the profile requires them.
- [ ] Reject protocol corruption and handle invalidation safely; ensure reset/free cannot reuse
  backend resources while valid work still references them.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- State-machine tests cover rerecording, resubmission, secondary inheritance, pending lifetime,
  failed recording, and pool destruction with independently specified expected outcomes.
- Run cargo test -p emulator --lib virgl_queue --quiet to preserve existing ordering; record new
  Venus state tests and browser command reuse evidence.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
