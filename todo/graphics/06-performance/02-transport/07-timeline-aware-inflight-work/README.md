# P07 — Experiment with timeline-aware in-flight work

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: P07
Depends: P03, R05, R06, R07, R08
Evidence: pending

Prerequisite lists: [P03](../../01-measurement/03-matched-baseline-runs/README.md), [R05](../../../01-shared-runtime/02-execution/01-coherence/README.md), [R06](../../../01-shared-runtime/02-execution/02-fences/README.md), [R07](../../../01-shared-runtime/02-execution/03-scheduler/README.md), [R08](../../../01-shared-runtime/02-execution/04-device-loss/README.md).

## Outcome

Independent work can overlap without violating guest fences, resource hazards or device-loss
behavior.

## Starting points

- [web/js/gpu-display.js](../../../../../web/js/gpu-display.js)
- [web/js/webgpu-3d.js](../../../../../web/js/webgpu-3d.js)
- [emulator/src/devices/virtio_gpu/tests/fence.rs](../../../../../emulator/src/devices/virtio_gpu/tests/fence.rs)
- [research/virtio-gpu-fence-timelines.md](../../../../../research/virtio-gpu-fence-timelines.md)

## Checklist

- [ ] Measure global GuestDisplay promise serialization and per-submit queue completion stalls.
- [ ] Prototype bounded in-flight encoding/submission with per-context/ring ordering and explicit
  cross-resource dependency edges.
- [ ] Coalesce safe completion waits while completing guest fences only when their required GPU work
  and guest-visible memory effects are finished.
- [ ] Exercise independent rings, same-ring reorder, shared resource hazards, rejected work, reset,
  context ID reuse and device loss.
- [ ] Compare queue utilization and P01 full workload results with the serial reference and record
  the selected concurrency limit.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run `cargo test -p emulator devices::virtio_gpu::tests::fence` and `node --test
  web/js/gpu-display-3d.test.mjs web/js/gpu-display-reset.test.mjs`; add real async hazard tests.
- No guest fence or readback becomes visible early, and failed/lost work cannot leave a wait hanging
  or revive old resources.
- Throughput gains include guest completion/input measurements and do not depend on allowing more
  uncompleted queued frames.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
