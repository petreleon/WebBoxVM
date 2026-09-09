# S12 — Cache compiled shaders and pipelines safely

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S12
Depends: S05, R02
Evidence: pending

Prerequisite lists: [S05](../../02-backend/01-wgsl-emission/README.md), [R02](../../../01-shared-runtime/01-resources/02-object-identity/README.md).

## Outcome

Repeated shaders reuse validated artifacts without stale device or binding state.

## Starting points

- [web/js/webgpu-virgl-draw.js](../../../../../web/js/webgpu-virgl-draw.js)
- [web/js/webgpu-3d-resources.js](../../../../../web/js/webgpu-3d-resources.js)
- [web/js/webgpu-session.js](../../../../../web/js/webgpu-session.js)

## Checklist

- [ ] Key caches by shader IR, layouts, render state, backend/compiler version and adapter/device
  generation.
- [ ] Deduplicate in-flight compilation and bound resident cache memory with deterministic eviction.
- [ ] Test cache collision handling, eviction, failed compilation and device-loss invalidation.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Repeated identical draws compile once, while a changed shader/layout/state produces a distinct
  valid pipeline.
- Cold/warm measurements record compilation count and first-frame latency without altering pixels.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
