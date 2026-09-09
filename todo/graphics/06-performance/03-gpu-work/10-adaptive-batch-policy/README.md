# P10 — Experiment with latency-budgeted adaptive batching

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P10
Depends: P03, P04, P07, P09
Evidence: pending

Prerequisite lists: [P03](../../01-measurement/03-matched-baseline-runs/README.md), [P04](../../02-transport/04-bounded-queue-draining/README.md), [P07](../../02-transport/07-timeline-aware-inflight-work/README.md), [P09](../09-pass-coalescing-state-reuse/README.md).

## Outcome

A bounded adaptive batching policy balances throughput and interactive latency from measured queue
pressure.

## Starting points

- [web/js/vm-worker/gpu-3d.js](../../../../../web/js/vm-worker/gpu-3d.js)
- [web/js/vm-worker/state.js](../../../../../web/js/vm-worker/state.js)
- [web/js/gpu-display.js](../../../../../web/js/gpu-display.js)
- [web/js/webgpu-virgl-material-batch.js](../../../../../web/js/webgpu-virgl-material-batch.js)

## Checklist

- [ ] Use P02 traces to identify batch-size versus fence/input latency tradeoffs for static,
  animated and interactive workloads.
- [ ] Prototype a policy with fixed byte/count/age caps that flushes at dependencies, explicit
  waits, present boundaries and latency deadlines.
- [ ] Keep a deterministic fixed-policy override and reset policy history on VM/device generation
  changes.
- [ ] Sweep policy parameters on training workloads then evaluate the frozen policy on the P01
  held-out corpus.
- [ ] Record worst-case queue residence, completion latency and memory; keep the adaptive policy
  only if it meets every required P01 budget.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Saturation and sparse interactive cases prove finite wait, bounded growth and exact ordering; no
  latency-sensitive work waits indefinitely for a full batch.
- Held-out matched native/browser results include p95 and tail outliers; mean FPS alone cannot
  select the policy.
- Fixed and adaptive modes produce equivalent API outputs and completions under the same
  deterministic workload.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
