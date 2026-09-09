# R02 — Unify generation-safe GPU object identity

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: R02
Depends: F05
Evidence: pending

Prerequisite lists: [F05](../../../00-foundation/02-reproducibility/02-check-runner/README.md).

## Outcome

Contexts, resources and asynchronous jobs cannot refer to reused or foreign objects.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/context.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/context.rs)
- [emulator/src/devices/virtio_gpu/resource/lifecycle.rs](../../../../../emulator/src/devices/virtio_gpu/resource/lifecycle.rs)
- [web/js/webgpu-3d-resources.js](../../../../../web/js/webgpu-3d-resources.js)

## Checklist

- [ ] Define protocol-independent context/resource handles with VM and device generations.
- [ ] Track attachment, reference lifetime, byte budgets and deferred destruction until dependent
  work completes.
- [ ] Reject cross-context access, double release, ID reuse with pending work and budget overflow
  transactionally.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Lifecycle tests cover destroy-before-completion, reset, ID reuse and allocation rollback.
- No browser GPU object survives its owning generation or leaks after repeated create/destroy loops.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
