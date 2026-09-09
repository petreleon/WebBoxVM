# R03 — Implement checked buffer ranges and sparse transfers

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: R03
Depends: R02
Evidence: pending

Prerequisite lists: [R02](../02-object-identity/README.md).

## Outcome

General buffer uploads and readbacks preserve exact guest bytes across sparse pages.

## Starting points

- [emulator/src/devices/virtio_gpu/backing.rs](../../../../../emulator/src/devices/virtio_gpu/backing.rs)
- [emulator/src/devices/virtio_gpu/three_d/transfer.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/transfer.rs)
- [emulator/src/devices/virtio_gpu/blob/transfer.rs](../../../../../emulator/src/devices/virtio_gpu/blob/transfer.rs)

## Checklist

- [ ] Represent byte ranges, alignment, backing pages and usage independently of current draw
  shapes.
- [ ] Implement subrange upload/download and dirty-range accounting without assuming contiguous
  guest RAM.
- [ ] Validate overflow, overlap policy, detached backing and copy alignment before modifying state.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Random page-spanning transfers match a byte-array oracle including partial final pages.
- Malformed range tests leave memory, resource budget and completion state unchanged.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
