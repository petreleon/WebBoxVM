# R08 — Recover or fail correctly after device loss

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: R08
Depends: R05, R06
Evidence: pending

Prerequisite lists: [R05](../01-coherence/README.md), [R06](../02-fences/README.md).

## Outcome

GPU loss invalidates every dependent object and unblocks the guest with a truthful outcome.

## Starting points

- [web/js/webgpu-session.js](../../../../../web/js/webgpu-session.js)
- [web/js/webgpu-errors.js](../../../../../web/js/webgpu-errors.js)
- [emulator/src/devices/virtio_gpu/reset.rs](../../../../../emulator/src/devices/virtio_gpu/reset.rs)

## Checklist

- [ ] Specify rebuildable CPU-backed state versus unrecoverable GPU-only state.
- [ ] Recreate valid resources/caches after device loss or return the API-defined
  device/context-lost result.
- [ ] Invalidate old callbacks and test repeated loss during upload, draw, readback and
  presentation.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Loss injection leaves no indefinitely pending descriptor/fence or old-generation rendering.
- Recovered images match the reference when recovery is allowed; failure paths return the expected
  guest error.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
