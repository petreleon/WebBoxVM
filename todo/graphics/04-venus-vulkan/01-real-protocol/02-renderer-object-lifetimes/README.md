# V02 — Implement Venus renderer object identity and reply/error ownership

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V02
Depends: V01, R02
Evidence: pending

Prerequisite lists: [V01](../01-wire-codec-generation/README.md), [R02](../../../01-shared-runtime/01-resources/02-object-identity/README.md).

## Outcome

Renderer handles and reply objects have validated type, context, generation, and lifetime semantics.

## Starting points

- [research/renderer-blob-ordering.md](../../../../../research/renderer-blob-ordering.md)
- [emulator/src/devices/virtio_gpu/three_d/context.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/context.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/blob.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/blob.rs)

## Checklist

- [ ] Implement the real protocol's object-ID allocation, create/destroy references, reply storage,
  and allocation-failure cleanup using the shared identity registry.
- [ ] Map protocol/Vulkan failures to the correct reply status and object outputs, including
  nullable handles and partial-creation cleanup.
- [ ] Reject stale/cross-context/type-confused handles and defined invalid transitions without
  leaking objects or consuming IDs on failed creation.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Initialization lifecycle fixtures include successful allocation/destruction, OOM, duplicate IDs,
  stale generations, and ordered replies with exact expected result codes.
- Run cargo test -p emulator --lib renderer_blob --quiet to preserve the old private ordering probe
  while proving the new Venus path does not interpret WBL1 as a real protocol command.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
