# F01 — Record the reproducible starting state

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: F01
Depends: none
Evidence: pending

## Outcome

A new worker can distinguish committed support, local edits, old claims, and fresh results.

## Starting points

- [todo.md](../../../../../todo.md)
- [research/virgl-compatibility.md](../../../../../research/virgl-compatibility.md)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Record git revision, dirty paths, current capsets and implemented feature limits in a short
  baseline receipt; preserve unrelated edits.
- [ ] Run the existing graphics suites and capture the first failing subcheck, including current
  matrix-texture work; reconcile stale prose against output.
- [ ] Record toolchain versions, available disk hashes and unavailable prerequisites; do not label
  old results as rerun.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run git status --short, cargo test -p emulator --lib virgl --quiet, and node --test
  web/js/*.test.mjs.
- Receipt separates observed tests from historical claims and identifies every unavailable check.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
