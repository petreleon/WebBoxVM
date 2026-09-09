# S06 — Implement shader interfaces and coordinate semantics

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S06
Depends: S03, S05
Evidence: pending

Prerequisite lists: [S03](../../01-frontends/03-tgsi-arithmetic/README.md), [S05](../01-wgsl-emission/README.md).

## Outcome

Vertex/fragment linkage preserves API-visible interpolation and coordinate conventions.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/context/vertex/layout.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/vertex/layout.rs)
- [web/js/webgpu-virgl-matrix.js](../../../../../web/js/webgpu-virgl-matrix.js)

## Checklist

- [ ] Map locations, builtins, interpolation modes and resource bindings across stages.
- [ ] Implement clip depth, origin, front-face and point/fragment coordinate differences explicitly.
- [ ] Add child tasks for remaining mandatory stages and fixed-function emulation identified by F04.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Off-center clipped triangles and flat/perspective interpolation match reference pixel probes.
- Interface mismatch, excessive varying counts and unsupported stages return explicit errors.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
