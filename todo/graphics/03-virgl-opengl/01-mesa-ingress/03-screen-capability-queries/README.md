# G03 — Implement Mesa screen capability negotiation from the shared registry

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G03
Depends: G01, R01, F03
Evidence: pending

Prerequisite lists: [G01](../01-capture-mesa-startup/README.md), [R01](../../../01-shared-runtime/01-resources/01-capability-registry/README.md), [F03](../../../00-foundation/01-contract/03-feature-matrix/README.md).

## Outcome

Mesa screen creation reads accurate VirGL1/VirGL2 limits and formats generated from implemented
features.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/capset.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/capset.rs)
- [emulator/src/devices/virtio_gpu/tests/virgl_draw_capset.rs](../../../../../emulator/src/devices/virtio_gpu/tests/virgl_draw_capset.rs)
- [emulator/src/devices/virtio_gpu/tests/virgl2.rs](../../../../../emulator/src/devices/virtio_gpu/tests/virgl2.rs)

## Checklist

- [ ] Map each queried VirGL1/VirGL2 field and growable-version boundary to the pinned upstream
  structure and shared capability registry.
- [ ] Implement only backed screen-query responses, including deterministic unknown-version and
  unsupported-format behavior.
- [ ] Replay Mesa screen startup after each implemented prerequisite; retain a named blocker if the
  requested profile needs a feature that is still absent.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Captured Mesa queries match byte-for-byte fixtures and every nonzero feature/limit points to an
  implementation test and mandatory profile requirement.
- Run cargo test -p emulator --lib virgl2 --quiet and cargo test -p emulator --lib virgl_draw_capset
  --quiet; successful capset discovery alone is not successful OpenGL initialization.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
