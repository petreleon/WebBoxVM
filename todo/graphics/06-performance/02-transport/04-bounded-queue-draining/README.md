# P04 — Experiment with event-driven bounded GPU packet draining

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P04
Depends: P03, R07
Evidence: pending

Prerequisite lists: [P03](../../01-measurement/03-matched-baseline-runs/README.md), [R07](../../../01-shared-runtime/02-execution/03-scheduler/README.md).

## Outcome

A measured experiment removes unnecessary polling latency while preserving bounded work and VM
responsiveness.

## Starting points

- [web/js/vm-worker/gpu-3d.js](../../../../../web/js/vm-worker/gpu-3d.js)
- [web/js/vm-worker/state.js](../../../../../web/js/vm-worker/state.js)
- [web/js/vm-worker/pump.js](../../../../../web/js/vm-worker/pump.js)
- [web/js/vm-worker/gpu-3d.test.mjs](../../../../../web/js/vm-worker/gpu-3d.test.mjs)

## Checklist

- [ ] Measure current one-packet-per-1000/60-ms poll behavior under multi-submit real API frames and
  establish queued-packet delay.
- [ ] Prototype a byte/count/time-bounded drain with an explicit reschedule signal and fair sharing
  with vCPU, UART, network and scanout work.
- [ ] Preserve backpressure, packet order, device generation, and reset handling; expose queue
  depth/latency in P02 counters.
- [ ] Run matched P01 workloads with old/new drain toggles, including CPU-heavy input and
  saturated-submit cases.
- [ ] Keep the variant only when completed-frame/input latency improves within the frozen regression
  and memory budgets; record negative experiment results.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run `node --test web/js/vm-worker/gpu-3d.test.mjs web/js/vm-worker/pump-responsive.test.mjs
  web/js/vm-worker/pump-network.test.mjs` and add burst/backpressure cases for the changed
  scheduler.
- No burst loses, duplicates or acknowledges a packet early; saturation remains bounded and
  UART/network/input service does not regress beyond P01 budgets.
- A/B results include complete guest-to-present timing rather than only packet extraction speed.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
