# Q02 — Close every mandatory compatibility requirement

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: Q02
Depends: I04, I05, I06, S11, F03, G15, V18
Evidence: pending

Prerequisite lists: [I04](../../../05-guest-validation/02-independent-validation/01-conformance-runner/README.md), [I05](../../../05-guest-validation/02-independent-validation/02-app-corpus/README.md), [I06](../../../05-guest-validation/02-independent-validation/03-browser-matrix/README.md), [S11](../../../02-shaders/04-validation/01-differential-corpus/README.md), [F03](../../../00-foundation/01-contract/03-feature-matrix/README.md), [G15](../../../03-virgl-opengl/05-guest-api-and-profile-proof/03-conformance-and-expansion/README.md), [V18](../../../04-venus-vulkan/06-stock-guest-and-profile-proof/03-conformance-and-expansion/README.md).

## Outcome

All promised profile rows have independent evidence; incomplete semantics cannot disappear from
scope.

## Starting points

- [research/virgl-compatibility.md](../../../../../research/virgl-compatibility.md)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)
- [README.md](../../../../../README.md)

## Checklist

- [ ] Reconcile generated mandatory feature inventories with implemented commands, limits and
  conformance results.
- [ ] Create nested child lists for every remaining gap, including advanced shader stages and
  version-specific requirements.
- [ ] Run complete required suites; publish supported/emulated/unsupported/blocked rows without
  relabeling failed mandatory behavior.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Zero unresolved mandatory rows, missing tests, unexpected skips or false advertised capabilities
  for each claimed profile.
- Earlier bring-up profiles do not complete the final OpenGL/GLES/Vulkan targets; a mandatory
  blocker leaves this task open.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
