# G15 — Close every mandatory GL profile gap with conformance evidence

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G15
Depends: F03, F05, G14, I02, I04
Evidence: pending

Prerequisite lists: [F03](../../../00-foundation/01-contract/03-feature-matrix/README.md), [F05](../../../00-foundation/02-reproducibility/02-check-runner/README.md), [G14](../02-real-egl-gl-context/README.md), [I02](../../../05-guest-validation/01-real-clients/02-mesa-opengl/README.md), [I04](../../../05-guest-validation/02-independent-validation/01-conformance-runner/README.md).

## Outcome

A reproducible GL/GLES compatibility report accounts for every mandatory requirement and links every
remaining expansion requirement to a concrete child task.

## Starting points

- [research/virgl-compatibility.md](../../../../../research/virgl-compatibility.md)
- [research/virgl2-capset.md](../../../../../research/virgl2-capset.md)
- [emulator/src/devices/virtio_gpu/three_d/capset.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/capset.rs)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Pin the applicable Piglit/dEQP/GL CTS test revisions, commands, must-pass selections,
  tolerances, and permitted environment skips for each target profile.
- [ ] Run the profile suites and classify pass/fail/unsupported/skip results; create small
  dependency-linked child tasks for every mandatory failure and broader requested feature, including
  transform feedback, advanced stages, compute, storage/image operations, or indirect drawing as
  applicable.
- [ ] Keep incomplete profiles and expansion targets visibly open; only expose an API
  version/extension after its mandatory semantics and guest-browser tests pass.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- The report includes raw logs, exact commands, suite/guest/browser/GPU revisions, case counts, and
  a requirement-to-test matrix with no unexplained mandatory skip or failure.
- Run make test and make web-pkg for release candidates; this task cannot be checked complete merely
  because a triangle demo passes or a future-feature backlog exists.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
