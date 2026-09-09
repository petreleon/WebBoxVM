# Q05 — Publish the supported matrix and reproducible demo

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: Q05
Depends: Q01, Q02, Q03, Q04
Evidence: pending

Prerequisite lists: [Q01](../../01-release-evidence/01-adversarial/README.md), [Q02](../../01-release-evidence/02-feature-closure/README.md), [Q03](../../01-release-evidence/03-maintainability/README.md), [Q04](../01-performance-acceptance/README.md).

## Outcome

A reviewer can reproduce compatibility and performance without relying on claims in prose.

## Starting points

- [README.md](../../../../../README.md)
- [research/virgl-compatibility.md](../../../../../research/virgl-compatibility.md)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)

## Checklist

- [ ] Write small linked API-support, architecture, limitations and reproduction documents from
  final receipts.
- [ ] Provide pinned guest/demo inputs, expected output and native/browser benchmark commands.
- [ ] Check redistribution licenses and describe test coverage accurately without claiming Khronos
  certification from local passes.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A clean-checkout walkthrough reproduces the published demo and evidence report.
- Every public compatibility/performance statement links to the exact profile, revision and
  measurement.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
