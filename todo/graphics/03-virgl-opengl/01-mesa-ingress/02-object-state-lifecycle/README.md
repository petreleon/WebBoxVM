# G02 — Accept the profile's VirGL object create, bind, and destroy transitions

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G02
Depends: G01, R02
Evidence: pending

Prerequisite lists: [G01](../01-capture-mesa-startup/README.md), [R02](../../../01-shared-runtime/01-resources/02-object-identity/README.md).

## Outcome

The first Mesa object-state trace replays transactionally with context-local handles and correct
unbind/destruction behavior.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/context.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream.rs)

## Checklist

- [ ] Inventory object types and lifecycle transitions used by the pinned profile and split
  remaining large object families into explicit child tasks before implementing them.
- [ ] Decode one complete create/bind/destroy lifecycle per child through shared context/resource
  identity instead of adding demo-specific special cases.
- [ ] Reject unknown handles, truncated objects, cross-context references, and invalid rebinds
  without partially committing the submitted stream.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Replay the captured object-lifecycle trace and assert exact final bindings, destruction effects,
  and unchanged state after each malformed variant.
- Run cargo test -p emulator --lib virgl --quiet and record the focused lifecycle test names and
  counts.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
