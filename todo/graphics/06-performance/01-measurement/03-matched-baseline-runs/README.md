# P03 — Collect reproducible native and browser baselines

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P03
Depends: P01, P02, I02, I03, I05, I06
Evidence: pending

Prerequisite lists: [P01](../01-native-comparison-contract/README.md), [P02](../02-stage-and-resource-timing/README.md), [I02](../../../05-guest-validation/01-real-clients/02-mesa-opengl/README.md), [I03](../../../05-guest-validation/01-real-clients/03-mesa-vulkan/README.md), [I05](../../../05-guest-validation/02-independent-validation/02-app-corpus/README.md), [I06](../../../05-guest-validation/02-independent-validation/03-browser-matrix/README.md).

## Outcome

Immutable raw evidence establishes each declared workload's baseline and largest measured
bottleneck.

## Starting points

- [Makefile](../../../../../Makefile)
- [scripts/virgl_guest_transport_smoke.sh](../../../../../scripts/virgl_guest_transport_smoke.sh)
- [research/webgpu-acceleration.md](../../../../../research/webgpu-acceleration.md)
- [web/js/gpu-display-diagnostics.js](../../../../../web/js/gpu-display-diagnostics.js)

## Checklist

- [ ] Build fresh serial and threaded Wasm with `make web-pkg`; identify all source, image, binary,
  asset and configuration hashes in the run manifest.
- [ ] Run both modes and the matched native API workload under P01 repetition and correctness rules;
  use `make web-benchmark` only as installed-disk serving support.
- [ ] Capture hardware adapter/renderer selection, guest renderer strings, validation errors, output
  checks and observed fallback boundaries on every run.
- [ ] Store raw per-frame samples plus p50/p95, throughput, memory, input latency and confidence
  intervals; include failed and timed-out trials.
- [ ] Rank vCPU execution, 60 Hz packet polling, copy/readback, shader compilation, queue waits and
  GPU work by measured share; keep existing transport demo numbers separately labeled.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- All P01 target workloads have matched complete results or explicit blocked entries; neither native
  transport smoke nor host replay substitutes for real guest API execution.
- Raw samples recompute the published ratios and quantify serial/threaded differences; cold boot and
  shader warmup are reported separately without removing steady-state guest CPU cost.
- No baseline is marked passing with a fallback adapter, renderer mismatch, missing correctness
  check, stale Wasm or uncompleted submissions.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
