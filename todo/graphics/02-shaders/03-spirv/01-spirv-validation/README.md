# S08 — Validate SPIR-V inputs and declared capabilities

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S08
Depends: S01, F02
Evidence: pending

Prerequisite lists: [S01](../../01-frontends/01-typed-ir/README.md), [F02](../../../00-foundation/01-contract/02-upstream-pins/README.md).

## Outcome

Vulkan shader binaries have a pinned validation environment and bounded decoder.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/shader.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/shader.rs)
- [Cargo.toml](../../../../../Cargo.toml)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)

## Checklist

- [ ] Integrate or generate a SPIR-V decoder/validator with pinned grammar and module limits.
- [ ] Validate entry points, decorations, memory model, capabilities and specialization constants.
- [ ] Keep unsupported optional capabilities disabled and track mandatory ones as open feature rows.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Pinned valid modules pass and malformed IDs/types/control flow fail before resource allocation.
- Fuzz inputs cannot panic, allocate beyond budget or hang compilation.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
