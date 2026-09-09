# G01 — Capture a reproducible stock Mesa VirGL startup trace

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G01
Depends: F02, F05, I01
Evidence: pending

Prerequisite lists: [F02](../../../00-foundation/01-contract/02-upstream-pins/README.md), [F05](../../../00-foundation/02-reproducibility/02-check-runner/README.md), [I01](../../../05-guest-validation/01-real-clients/01-guest-image/README.md).

## Outcome

A pinned Mesa guest probe produces a protocol trace identifying the first unsupported
screen-initialization operation.

## Starting points

- [guest/virgl-clear-demo/README.md](../../../../../guest/virgl-clear-demo/README.md)
- [scripts/virgl_guest_transport_smoke.sh](../../../../../scripts/virgl_guest_transport_smoke.sh)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode.rs)

## Checklist

- [ ] Build the pinned upstream Mesa VirGL driver into a reproducible guest fixture and record
  package/source hashes and build options.
- [ ] Run one minimal EGL display/config probe and capture guest stderr plus ordered context,
  capset, resource, and VirGL requests.
- [ ] Name the first failing request and preserve its exact bytes as a regression fixture; classify
  a boot timeout separately from a renderer failure.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- The evidence includes Mesa revision, guest kernel/image hash, driver-loading diagnostics, and the
  first unsupported operation or successful probe result; a freestanding handcrafted VirGL demo
  cannot satisfy this task.
- Run make -C guest/virgl-clear-demo and cargo test -p emulator --lib virgl --quiet to preserve the
  existing transport and parser baseline.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
