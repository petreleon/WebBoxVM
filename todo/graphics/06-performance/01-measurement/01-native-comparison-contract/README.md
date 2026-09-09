# P01 — Freeze native comparison and performance targets

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P01
Depends: F01, F02, F03, F04, F05, I01
Evidence: pending

Prerequisite lists: [F01](../../../00-foundation/01-contract/01-baseline/README.md), [F02](../../../00-foundation/01-contract/02-upstream-pins/README.md), [F03](../../../00-foundation/01-contract/03-feature-matrix/README.md), [F04](../../../00-foundation/02-reproducibility/01-feasibility/README.md), [F05](../../../00-foundation/02-reproducibility/02-check-runner/README.md), [I01](../../../05-guest-validation/01-real-clients/01-guest-image/README.md).

## Outcome

A preregistered, versioned protocol defines what near-native means before optimization; initial
thresholds are proposed defaults until this task freezes them.

## Starting points

- [research/webgpu-acceleration.md](../../../../../research/webgpu-acceleration.md)
- [Makefile](../../../../../Makefile)
- [web/js/gpu-display-diagnostics.js](../../../../../web/js/gpu-display-diagnostics.js)

## Checklist

- [ ] Select native reference using the same physical GPU, equivalent API
  workload/assets/output/settings, and pinned native/browser driver and Mesa versions; classify
  unavoidable stack differences.
- [ ] Propose then freeze >=80% native completed-work throughput, p95 completed frame time <=1.25x
  native, and p95 input-to-present latency <=native plus one display refresh interval for every
  declared target workload.
- [ ] Freeze sample counts, warmup, cold/warm runs, repetition/randomization, refresh rate, vsync
  policy, statistical intervals, power state, thermal controls and API correctness thresholds.
- [ ] Define primary end-to-end native ratio including guest API, emulated CPU, transport and GPU
  work; preserve GPU-only, host replay and software A/B as secondary diagnostics.
- [ ] Record required hardware/API profile availability; unavailable comparable native hardware or
  unsupported required semantics block a claim instead of changing the workload or denominator.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Protocol records explicit proposed-to-frozen dated version state before P03 or any optimization;
  numerical thresholds and workload list cannot be relaxed after observing results.
- Each declared API/profile has a runnable native comparator and matched browser guest
  configuration, or an explicit blocker; faster queue.submit alone never meets acceptance.
- Comparison requires correct output, actual hardware GPU/renderer evidence and no hidden software
  fallback on both sides.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
