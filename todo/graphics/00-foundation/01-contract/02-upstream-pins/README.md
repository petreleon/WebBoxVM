# F02 — Pin upstream protocol and test inputs

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: F02
Depends: F01
Evidence: pending

Prerequisite lists: [F01](../01-baseline/README.md).

## Outcome

Existing protocol layouts and reference tests are reproducible instead of following mutable upstream
main. F02.4 extends that foundation with the target-profile sources discovered by F03.1.

## Starting points

- [research/virgl2-capset.md](../../../../../research/virgl2-capset.md)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)
- [Cargo.toml](../../../../../Cargo.toml)

## Checklist

- [x] [F02.1 — Define the immutable graphics-input inventory](01-input-inventory/README.md)
- [x] [F02.2 — Verify isolated source fetches and hashes](02-fetch-verifier/README.md)
- [x] [F02.3 — Bind ABI fixtures and generators to the manifest](03-provenance-contract/README.md)
- [ ] [F02.4 — Pin target-profile normative sources and conformance manifests](04-profile-source-inputs/README.md)

## Verification

- A clean temporary build fetches the pinned inputs and detects a deliberately wrong hash.
- Every ABI fixture and future code generator records the canonical inventory-lock revision.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

The prior [F02 receipt](evidence.md) remains historical evidence for the 17-input F02.1–F02.3
closure only. It is not evidence that the six F03.1 profile sources are pinned.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
