# Q04 — Approve measured near-native guest graphics results

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: Q04
Depends: Q02, I06, P15
Evidence: pending

Prerequisite lists: [Q02](../../01-release-evidence/02-feature-closure/README.md), [I06](../../../05-guest-validation/02-independent-validation/03-browser-matrix/README.md), [P15](../../../06-performance/04-execution-and-proof/15-near-native-claim-audit/README.md).

## Outcome

The performance claim is backed by the preregistered complete guest workload matrix.

## Starting points

- [research/webgpu-acceleration.md](../../../../../research/webgpu-acceleration.md)
- [README.md](../../../../../README.md)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Re-run native and browser-guest benchmarks on the same recorded hardware under frozen P01
  conditions.
- [ ] Check throughput, p95 frame time and input latency per workload using confidence intervals,
  not a favorable aggregate.
- [ ] Keep slow, emulated, skipped and incompatible workloads visible; leave the goal incomplete if
  a required threshold fails.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Every mandatory workload meets the frozen thresholds with correct output and verified hardware
  acceleration.
- No submit-only timing, software comparison or cross-machine ratio substitutes for the native
  comparison.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
