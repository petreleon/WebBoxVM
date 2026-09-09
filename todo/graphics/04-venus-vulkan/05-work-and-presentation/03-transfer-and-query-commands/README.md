# V15 — Implement remaining mandatory transfer and query command families

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V15
Depends: V09, R03, R04, R06
Evidence: pending

Prerequisite lists: [V09](../../03-command-and-sync-state/03-timeline-and-memory-dependencies/README.md), [R03](../../../01-shared-runtime/01-resources/03-buffer-transfers/README.md), [R04](../../../01-shared-runtime/01-resources/04-texture-layout/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md).

## Outcome

The target Vulkan profile has explicit independently executable children for every remaining
transfer/query command family.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/residency/copy.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/residency/copy.rs)
- [emulator/src/devices/virtio_gpu/resource_transfer/readback.rs](../../../../../emulator/src/devices/virtio_gpu/resource_transfer/readback.rs)
- [web/js/webgpu-readback.js](../../../../../web/js/webgpu-readback.js)

## Checklist

- [ ] Inventory mandatory buffer/image copy, fill/update, blit/resolve, and query-pool operations
  against the profile and expand this list into one child per operation family before
  implementation.
- [ ] Implement each child with exact format/stride/region/alignment and query
  availability/result-copy semantics through shared resources.
- [ ] Reject malformed ranges and unsupported required operations deterministically; do not
  fabricate timestamp precision or count results when the browser cannot supply required semantics.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Each child runs a guest-produced Vulkan command fixture and compares resulting bytes/pixels/query
  ordering with an independent implementation.
- The parent remains unchecked until all mandatory children pass; capability entries name exact
  supported formats/query types and any browser feature prerequisites.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
