# P15 — Audit reproducible near-native guest graphics claims

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P15
Depends: P01, P02, P03, P14, I04, I05, I06
Evidence: pending

Prerequisite lists: [P01](../../01-measurement/01-native-comparison-contract/README.md), [P02](../../01-measurement/02-stage-and-resource-timing/README.md), [P03](../../01-measurement/03-matched-baseline-runs/README.md), [P14](../14-portability-and-soak-regression/README.md), [I04](../../../05-guest-validation/02-independent-validation/01-conformance-runner/README.md), [I05](../../../05-guest-validation/02-independent-validation/02-app-corpus/README.md), [I06](../../../05-guest-validation/02-independent-validation/03-browser-matrix/README.md).

## Outcome

A reproducible claim audit states exactly which real guest APIs/workloads meet the frozen
near-native targets and leaves unmet requirements open.

## Starting points

- [README.md](../../../../../README.md)
- [research/webgpu-acceleration.md](../../../../../research/webgpu-acceleration.md)
- [research/virgl-compatibility.md](../../../../../research/virgl-compatibility.md)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)

## Checklist

- [ ] Rerun the frozen matched native/browser protocol from clean documented build inputs and
  selected optimization configuration.
- [ ] Recompute ratios and confidence intervals from archived raw samples, including failures,
  warmup policy, complete guest CPU/transport cost and hardware/fallback verification.
- [ ] Require each declared API/profile workload to pass correctness and all P01 thresholds; list
  capability, profile, CPU, driver or hardware blockers without changing denominators.
- [ ] Publish a compact support/performance matrix with exact versions, resolutions, quality, native
  comparator, raw evidence paths and rerun commands.
- [ ] Update public claims only for observed passing scope; preserve incomplete compatibility,
  conformance and performance tasks as unchecked.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A second execution from recorded inputs reproduces acceptance within P01 statistical rules; no
  microbenchmark, transport ack or synthetic replay is substituted.
- Each claimed workload meets >=80% native throughput, p95 <=1.25x native and input latency <=native
  plus one refresh if those proposed P01 defaults were frozen; any changed pre-baseline targets are
  explicitly recorded.
- The overall graphics goal cannot be marked complete with unmet declared API/profile or performance
  requirements; rejected optimizations and infeasible features are recorded without being renamed
  successes.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
