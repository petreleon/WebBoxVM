# Q03 — Audit module boundaries and reproducible builds

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: Q03
Depends: F06, I06
Evidence: pending

Prerequisite lists: [F06](../../../00-foundation/02-reproducibility/03-file-layout/README.md), [I06](../../../05-guest-validation/02-independent-validation/03-browser-matrix/README.md).

## Outcome

The complete implementation can be rebuilt and safely extended in small modules.

## Starting points

- [emulator/tests/architecture_boundaries.rs](../../../../../emulator/tests/architecture_boundaries.rs)
- [emulator/tests/source_file_limits.rs](../../../../../emulator/tests/source_file_limits.rs)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Check protocol/frontend/backend dependencies and document each public module contract.
- [ ] Rebuild pinned generators and Wasm packages from a clean checkout; compare reproducible
  outputs.
- [ ] Audit all maintained code, tests, scripts and documentation for the 180-line limit without
  compressed one-line workarounds.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run make test, make web-pkg and python3 scripts/check_graphics_roadmap.py with clean result
  receipts.
- All new/touched files satisfy the limit and generated artifacts can be regenerated from pinned
  inputs.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
