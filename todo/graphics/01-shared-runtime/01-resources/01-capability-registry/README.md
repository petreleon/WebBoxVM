# R01 — Build a single tested capability registry

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: R01
Depends: F03, F04
Evidence: pending

Prerequisite lists: [F03](../../../00-foundation/01-contract/03-feature-matrix/README.md), [F04](../../../00-foundation/02-reproducibility/01-feasibility/README.md).

## Outcome

Guest queries reflect the intersection of implemented semantics and the active browser adapter.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/capset.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/capset.rs)
- [emulator/src/devices/virtio_gpu/feature.rs](../../../../../emulator/src/devices/virtio_gpu/feature.rs)
- [web/js/webgpu-session.js](../../../../../web/js/webgpu-session.js)

## Checklist

- [ ] Represent feature support, implementation path and runtime adapter limits in a shared
  versioned registry.
- [ ] Generate or validate VirGL/Venus advertised fields from tested entries; keep private capsets
  distinct.
- [ ] Reject unsupported requests and test adapter-limit downgrade without exposing stale
  capabilities.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Exact capset bytes match golden fixtures from pinned headers.
- Each enabled field has positive and disabled-adapter negative tests; run cargo test -p emulator
  --lib capset --quiet.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
