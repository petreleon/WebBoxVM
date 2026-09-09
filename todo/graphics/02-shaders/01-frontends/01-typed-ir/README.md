# S01 — Define a typed shader intermediate representation

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S01
Depends: F02, F04
Evidence: pending

Prerequisite lists: [F02](../../../00-foundation/01-contract/02-upstream-pins/README.md), [F04](../../../00-foundation/02-reproducibility/01-feasibility/README.md).

## Outcome

TGSI and SPIR-V share explicit scalar types, control flow and resource semantics.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/shader.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/shader.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse.rs)
- [web/js/webgpu-virgl-material-batch-shaders.js](../../../../../web/js/webgpu-virgl-material-batch-shaders.js)

## Checklist

- [ ] Define typed values, basic blocks, entry points, interfaces and resource bindings with stable
  serialization.
- [ ] Add an IR validator for types, dominance, binding bounds and stage restrictions.
- [ ] Create a tiny reference evaluator for the initial arithmetic subset and precise
  unsupported-operation diagnostics.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Valid arithmetic fixtures survive round trip; malformed SSA/type/control-flow fixtures are
  rejected.
- Validation terminates within explicit instruction/block budgets.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
