# P12 — Remove avoidable readbacks from the frame hot path

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P12
Depends: P03, R05, R06, R08, P08
Evidence: pending

Prerequisite lists: [P03](../../01-measurement/03-matched-baseline-runs/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md), [R08](../../../01-shared-runtime/02-execution/04-device-loss/README.md), [P08](../../03-gpu-work/08-resident-rendergraph/README.md).

## Outcome

Guest-required readbacks remain correct while optional diagnostics and presentation checks avoid
synchronizing every frame.

## Starting points

- [web/js/webgpu-readback.js](../../../../../web/js/webgpu-readback.js)
- [web/js/webgpu-readback.test.mjs](../../../../../web/js/webgpu-readback.test.mjs)
- [emulator/src/devices/virtio_gpu/completion/readback.rs](../../../../../emulator/src/devices/virtio_gpu/completion/readback.rs)
- [research/virgl-gpu-readback.md](../../../../../research/virgl-gpu-readback.md)

## Checklist

- [ ] Classify each measured readback as required by guest API semantics, coherence fallback,
  presentation, or diagnostics.
- [ ] Move optional diagnostic/pixel sampling off the critical path with bounded sampling and tag
  samples to completed frame identities.
- [ ] Pipeline required asynchronous copies and reuse bounded readback buffers where legal; blocking
  guest maps/waits still complete only after bytes are available.
- [ ] Verify row padding, BGRA/RGBA conversion, partial transfer failure, outstanding readback
  teardown and device loss.
- [ ] Compare maps/frame, stall duration, memory and native ratios with the synchronized reference,
  including readback-heavy workloads.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run `node --test web/js/webgpu-readback.test.mjs web/js/gpu-virgl-resident-readback.test.mjs
  web/js/gpu-display-reset.test.mjs`.
- All guest-requested bytes and fence timing remain correct; asynchronous instrumentation cannot
  convert a failed or incomplete frame into success.
- Rendering gains are reported alongside readback-heavy results and P01 correctness sampling
  coverage is retained.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
