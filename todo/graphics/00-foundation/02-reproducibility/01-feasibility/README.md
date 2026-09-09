# F04 — Prove browser semantic mappings before promising profiles

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: F04
Depends: F03
Evidence: pending

Prerequisite lists: [F03](../../01-contract/03-feature-matrix/README.md).

## Outcome

Each API requirement has a sound browser implementation, a measurable emulation plan or a concrete
blocker.

## Starting points

- [research/venus-foundations.md](../../../../../research/venus-foundations.md)
- [research/webgpu-acceleration.md](../../../../../research/webgpu-acceleration.md)
- [emulator/src/devices/virtio_gpu/blob/map.rs](../../../../../emulator/src/devices/virtio_gpu/blob/map.rs)

## Checklist

- [ ] Write tiny probes for mapped-memory coherence, external memory/sync, geometry/tessellation,
  shader precision and mandatory limits; split each probe into a child task before implementation.
- [ ] Classify direct WebGPU mapping, exact emulation, optional exclusion or mandatory blocker;
  compare against stock Mesa behavior.
- [ ] Measure transfer/compute cost of emulation and record which profiles remain blocked; retain
  the browser-local execution objective.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Every probe has a reference output, expected failure and pinned browser/adapter record.
- No invented capability bit or custom guest protocol is accepted as proof of the standard feature.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
