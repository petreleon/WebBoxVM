# Q01 — Harden untrusted graphics inputs and resource isolation

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: Q01
Depends: I04, R08
Evidence: pending

Prerequisite lists: [I04](../../../05-guest-validation/02-independent-validation/01-conformance-runner/README.md), [R08](../../../01-shared-runtime/02-execution/04-device-loss/README.md).

## Outcome

Malformed guest graphics cannot escape budgets, leak another context or wedge the VM.

## Starting points

- [emulator/src/devices/virtio_gpu/queue.rs](../../../../../emulator/src/devices/virtio_gpu/queue.rs)
- [emulator/src/devices/virtio_gpu/backing.rs](../../../../../emulator/src/devices/virtio_gpu/backing.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream.rs)
- [web/js/webgpu-errors.js](../../../../../web/js/webgpu-errors.js)

## Checklist

- [ ] Create child fuzz targets for VirGL, Venus, TGSI, SPIR-V and resource metadata with
  deterministic seeds.
- [ ] Inject reset/loss during pending work and cross-context resource-ID misuse.
- [ ] Minimize crashes, leaks and timeouts; add regression fixtures and run bounded stress.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Recorded fuzz/stress budgets complete with zero unresolved crashes, hangs or cross-context leaks.
- Every found issue has a reproducer and a verified fixed revision.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
