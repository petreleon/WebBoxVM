# I04 — Integrate pinned independent conformance suites

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: I04
Depends: I01, F03, F05
Evidence: pending

Prerequisite lists: [I01](../../01-real-clients/01-guest-image/README.md), [F03](../../../00-foundation/01-contract/03-feature-matrix/README.md), [F05](../../../00-foundation/02-reproducibility/02-check-runner/README.md).

## Outcome

Compatibility reports contain exact executed, passed, failed and skipped cases.

## Starting points

- [Makefile](../../../../../Makefile)
- [scripts/virgl_guest_transport_smoke.sh](../../../../../scripts/virgl_guest_transport_smoke.sh)
- [research/virgl-compatibility.md](../../../../../research/virgl-compatibility.md)

## Checklist

- [ ] Add separate GL/GLES Piglit or dEQP and Vulkan CTS runners with pinned case lists and log
  parsers.
- [ ] Run identical selected cases on reference and guest targets, preserving full results and
  minimized reproductions.
- [ ] Make crash, timeout, missing case, unsupported mandatory case and unexpected skip fail the
  selected profile.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A deliberately skipped mandatory case and corrupted result both fail the runner.
- Reports distinguish a passing subset from complete target-profile coverage and formal
  certification.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
