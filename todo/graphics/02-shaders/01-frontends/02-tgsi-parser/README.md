# S02 — Replace TGSI shape matching with a real parser

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S02
Depends: S01
Evidence: pending

Prerequisite lists: [S01](../01-typed-ir/README.md).

## Outcome

Pinned Mesa-produced TGSI is parsed structurally, including declarations and chunked input.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/context/shader/chunks.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/context/shader/chunks.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/shader.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode/shader.rs)

## Checklist

- [ ] Parse the pinned VirGL shader wire representation, declarations, operands, swizzles, modifiers
  and immediates.
- [ ] Preserve chunk ordering and source locations; lower syntax to typed IR without recognizing
  whole shader strings.
- [ ] Add captured Mesa shaders plus malformed/truncated/chunk-overlap fixtures.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Two semantically equivalent differently spelled shaders produce equivalent IR.
- Unrecognized instructions produce a bounded explicit error, never a guessed known shape.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
