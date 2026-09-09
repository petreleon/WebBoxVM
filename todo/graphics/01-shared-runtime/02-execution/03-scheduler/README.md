# R07 — Build a bounded shared submission scheduler

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: R07
Depends: R06
Evidence: pending

Prerequisite lists: [R06](../02-fences/README.md).

## Outcome

Both guest protocols share backpressure and hazard tracking without changing program order.

## Starting points

- [web/js/vm-worker/gpu-3d.js](../../../../../web/js/vm-worker/gpu-3d.js)
- [web/js/vm-worker/state.js](../../../../../web/js/vm-worker/state.js)
- [web/js/gpu-display.js](../../../../../web/js/gpu-display.js)

## Checklist

- [ ] Introduce a protocol-neutral validated work queue with resource dependencies and bounded
  bytes/jobs.
- [ ] Drain eligible work and preserve context ordering; distinguish queue saturation from permanent
  protocol errors.
- [ ] Test multiple contexts, stalled consumers, reset during backlog and deterministic fairness.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Submitted/completed/failed counters balance after stress and drain.
- Bounded scheduling cannot starve guest input or reorder conflicting resource accesses.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
