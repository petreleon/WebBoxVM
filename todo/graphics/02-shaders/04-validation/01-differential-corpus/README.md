# S11 — Build an independent shader differential corpus

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S11
Depends: S04, S07, S09, S10, F05
Evidence: pending

Prerequisite lists: [S04](../../01-frontends/04-tgsi-control/README.md), [S07](../../02-backend/03-texture-operations/README.md), [S09](../../03-spirv/02-spirv-lowering/README.md), [S10](../../03-spirv/03-storage-compute/README.md), [F05](../../../00-foundation/02-reproducibility/02-check-runner/README.md).

## Outcome

Both shader frontends have independent differential coverage before full guest integration.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse.rs)
- [guest/virgl-clear-demo/README.md](../../../../../guest/virgl-clear-demo/README.md)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Import licensed minimized real shaders and deterministic generated programs from pinned
  sources.
- [ ] Run native reference and standalone browser shader-replay harnesses for both frontends with
  exact integer/defined float comparisons; full guest-route checks belong to I02/I03.
- [ ] Minimize failures into small permanent fixtures and record seed, source hash and compiler
  diagnostics.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Corpus command fails on a deliberately perturbed output and rejects missing reference results.
- Every required shader feature has positive, boundary and negative coverage.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
