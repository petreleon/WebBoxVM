# P14 — Validate optimization portability and sustained operation

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P14
Depends: P04, P05, P06, P07, P08, P09, P10, P11, P12, P13, I04, I06
Evidence: pending

Prerequisite lists: [P04](../../02-transport/04-bounded-queue-draining/README.md), [P05](../../02-transport/05-worker-transfer-ring/README.md), [P06](../../02-transport/06-dirty-range-resource-transfer/README.md), [P07](../../02-transport/07-timeline-aware-inflight-work/README.md), [P08](../../03-gpu-work/08-resident-rendergraph/README.md), [P09](../../03-gpu-work/09-pass-coalescing-state-reuse/README.md), [P10](../../03-gpu-work/10-adaptive-batch-policy/README.md), [P11](../../03-gpu-work/11-gpu-driven-command-experiment/README.md), [P12](../12-asynchronous-readback-policy/README.md), [P13](../13-guest-cpu-submission-efficiency/README.md), [I04](../../../05-guest-validation/02-independent-validation/01-conformance-runner/README.md), [I06](../../../05-guest-validation/02-independent-validation/03-browser-matrix/README.md).

## Outcome

Selected optimizations survive sustained real API use across the declared browser/hardware matrix
and failure conditions.

## Starting points

- [Makefile](../../../../../Makefile)
- [web/js/gpu-display-reset.test.mjs](../../../../../web/js/gpu-display-reset.test.mjs)
- [web/js/gpu-display-ownership.test.mjs](../../../../../web/js/gpu-display-ownership.test.mjs)
- [web/js/webgpu-errors.test.mjs](../../../../../web/js/webgpu-errors.test.mjs)

## Checklist

- [ ] Freeze selected optimization toggles and run I04 conformance plus I05 workloads on each
  declared I06 platform/mode.
- [ ] Perform sustained allocation/draw/readback cycles, context churn, resize, suspend/resume and
  device-loss/reset trials with bounded watchdogs.
- [ ] Track resident/cached/readback memory, queue depth, errors and throughput over time; compare
  first and last steady windows.
- [ ] Exercise unsupported adapters/features and disabled optimizations to verify honest
  capability/fallback reporting.
- [ ] Record platform-specific regressions as blockers or disable the optimization on that platform
  without weakening the declared compatibility target.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run `make test` and `make web-pkg`; then collect real-browser evidence because Node fake-device
  tests do not prove hardware execution.
- No unbounded resource growth, stale output, missed fence or hung completion occurs in the frozen
  soak interval; recovery semantics match R08.
- Every claimed target platform satisfies P01 correctness/performance thresholds with selected
  optimizations, or is explicitly unresolved.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
