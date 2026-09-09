# V05 — Implement Vulkan mapped-memory visibility and flush/invalidate ranges

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V05
Depends: V04, R05, R06
Evidence: pending

Prerequisite lists: [V04](../01-memory-allocation-binding/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md).

## Outcome

Guest mapped memory observes the declared coherent/noncoherent rules across host writes and GPU
reads/writes.

## Starting points

- [emulator/src/devices/virtio_gpu/blob/map.rs](../../../../../emulator/src/devices/virtio_gpu/blob/map.rs)
- [emulator/src/devices/virtio_gpu/blob/transfer.rs](../../../../../emulator/src/devices/virtio_gpu/blob/transfer.rs)
- [emulator/src/devices/virtio_gpu/resource_transfer/readback.rs](../../../../../emulator/src/devices/virtio_gpu/resource_transfer/readback.rs)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)

## Checklist

- [ ] Map guest-visible allocation ranges onto shared resource storage with precise map offsets,
  sizes, lifetime, and nonCoherentAtomSize behavior.
- [ ] Implement flush and invalidate visibility transitions without treating sparse guest RAM as
  contiguous importable GPU memory.
- [ ] Reject invalid ranges and use-after-unmap/free; advertise HOST_COHERENT only when its full
  observable behavior is implemented.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A guest program writes mapped bytes, flushes when required, copies/uses them on GPU, then waits
  and invalidates to verify GPU-written bytes at nonzero offsets.
- Run cargo test -p emulator --lib default_blob --quiet and cargo test -p emulator --lib host_blob
  --quiet; record actual selected test names/counts and add missing Vulkan-specific coherence cases.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
