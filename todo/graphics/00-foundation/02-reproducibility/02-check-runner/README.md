# F05 — Create a reproducible graphics verification runner

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: F05
Depends: F01, F02
Evidence: pending

Prerequisite lists: [F01](../../01-contract/01-baseline/README.md), [F02](../../01-contract/02-upstream-pins/README.md).

## Outcome

Each future leaf can name an exact one-command check and produce an inspectable receipt.

## Starting points

- [Makefile](../../../../../Makefile)
- [scripts/serve_web.py](../../../../../scripts/serve_web.py)
- [scripts/virgl_guest_transport_smoke.sh](../../../../../scripts/virgl_guest_transport_smoke.sh)
- [emulator/tests/source_file_limits.rs](../../../../../emulator/tests/source_file_limits.rs)

## Checklist

- [ ] Create scripts/graphics/check.py with named device, browser, wasm, guest and conformance
  checks in small helper modules.
- [ ] Record command, revision, dirty diff hash, tool versions, duration, exit status, expected test
  count and artifact hashes; fail on zero matched tests.
- [ ] Make missing assets/browser/native hardware explicit blocked results; add a documented command
  per leaf before checking it complete.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- The runner propagates a deliberately failing child command and rejects empty test selection.
- Existing make test and make web-pkg are runnable lanes; a native guest smoke is labeled
  transport-only.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
