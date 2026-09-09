# F02 — Pin upstream protocol and test inputs

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: F02
Depends: F01
Evidence: [receipt](evidence.md)

Prerequisite lists: [F01](../01-baseline/README.md).

## Outcome

Protocol layouts and reference tests are reproducible instead of following mutable upstream main.

## Starting points

- [research/virgl2-capset.md](../../../../../research/virgl2-capset.md)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)
- [Cargo.toml](../../../../../Cargo.toml)

## Checklist

- [x] [F02.1 — Define the immutable graphics-input inventory](01-input-inventory/README.md)
- [x] [F02.2 — Verify isolated source fetches and hashes](02-fetch-verifier/README.md)
- [x] [F02.3 — Bind ABI fixtures and generators to the manifest](03-provenance-contract/README.md)

## Verification

- A clean temporary build fetches the pinned inputs and detects a deliberately wrong hash.
- Every ABI fixture and future code generator records the canonical inventory-lock revision.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
