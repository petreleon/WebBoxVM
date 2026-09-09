# V03 — Implement actual Venus rings, replies, and protocol negotiation

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V03
Depends: V01, V02, R06
Evidence: pending

Prerequisite lists: [V01](../01-wire-codec-generation/README.md), [V02](../02-renderer-object-lifetimes/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md).

## Outcome

A stock pinned Mesa Venus guest exchanges initialization commands and replies over the actual
renderer protocol.

## Starting points

- [research/venus-foundations.md](../../../../../research/venus-foundations.md)
- [emulator/src/devices/virtio_gpu/three_d.rs](../../../../../emulator/src/devices/virtio_gpu/three_d.rs)
- [emulator/src/devices/virtio_gpu/three_d/pending.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/pending.rs)
- [emulator/src/devices/virtio_gpu/fence.rs](../../../../../emulator/src/devices/virtio_gpu/fence.rs)

## Checklist

- [ ] Implement negotiated protocol versions, ring creation/control, producer/consumer positions,
  synchronous replies, and required progress notifications.
- [ ] Use validated blob/ring storage with wraparound and backpressure; keep Venus protocol rings
  distinct from VirtIO fence timeline ring indices.
- [ ] Test stock Mesa startup in an explicit development fixture and preserve the first unsupported
  command; reject incompatible negotiation and lost-ring state without deadlock.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- An unmodified Mesa trace reaches the specified initialization checkpoint with correct replies;
  ring wraparound, full-buffer pressure, cancellation, and invalid offsets have deterministic
  results.
- Run cargo test -p emulator --lib fence --quiet; no public Venus capset or Vulkan device claim is
  enabled by a transport-only checkpoint.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
