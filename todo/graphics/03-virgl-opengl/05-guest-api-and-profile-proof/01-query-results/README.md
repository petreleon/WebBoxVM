# G13 — Implement VirGL query lifecycle and result availability

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G13
Depends: G02, R06, R07
Evidence: pending

Prerequisite lists: [G02](../../01-mesa-ingress/02-object-state-lifecycle/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md), [R07](../../../01-shared-runtime/02-execution/03-scheduler/README.md).

## Outcome

The selected profile's mandatory queries return correctly ordered availability/results and preserve
query lifetime.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode.rs)
- [emulator/src/devices/virtio_gpu/three_d/pending.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/pending.rs)
- [emulator/src/devices/virtio_gpu/fence.rs](../../../../../emulator/src/devices/virtio_gpu/fence.rs)

## Checklist

- [ ] Enumerate query types required by the profile and create one small child task per
  independently implemented query type.
- [ ] Implement create/begin/end/get-result/destroy state transitions with availability distinct
  from completion and result writes ordered after producers.
- [ ] Reject illegal nesting, unsupported query types, and stale handles; do not fabricate
  timestamps or sample counts from host wall time.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Guest tests observe not-ready then ready results, correct zero/nonzero reference cases, and
  destruction/reset while a result is pending.
- Run cargo test -p emulator --lib fence --quiet; every enabled query has a named deterministic
  state-machine test and actual browser result evidence.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
