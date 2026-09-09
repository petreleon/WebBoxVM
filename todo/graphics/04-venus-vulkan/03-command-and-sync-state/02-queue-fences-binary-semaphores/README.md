# V08 — Implement queue submission with binary semaphores and fences

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V08
Depends: V07, R06, R07
Evidence: pending

Prerequisite lists: [V07](../01-command-buffer-lifecycle/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md), [R07](../../../01-shared-runtime/02-execution/03-scheduler/README.md).

## Outcome

Guest waits and fence status reflect completed dependency-ordered GPU submissions without a global
wait after every draw.

## Starting points

- [emulator/src/devices/virtio_gpu/fence.rs](../../../../../emulator/src/devices/virtio_gpu/fence.rs)
- [emulator/src/devices/virtio_gpu/three_d/pending.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/pending.rs)
- [web/js/worker-vm/gpu3d-ack.js](../../../../../web/js/worker-vm/gpu3d-ack.js)

## Checklist

- [ ] Map queue-submit wait/signal semaphores and fence lifecycle to shared scheduling and
  completion primitives.
- [ ] Implement finite/infinite waits, reset/status, submit failure, and empty-submit behavior;
  retain ordering between guest-visible resource effects and completion.
- [ ] Test cancellation/device loss without hanging guest waits and distinguish execution completion
  from canvas presentation.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A guest producer/consumer test uses two submissions with a binary semaphore and verifies
  not-ready/ready fence states, timed waits, reset/reuse, and correct dependent readback.
- Run cargo test -p emulator --lib fence --quiet; instrumentation proves independent work is not
  forced through a per-draw onSubmittedWorkDone wait.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
