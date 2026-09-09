# S03 — Lower TGSI arithmetic by operation family

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S03
Depends: S02
Evidence: pending

Prerequisite lists: [S02](../02-tgsi-parser/README.md).

## Outcome

Arithmetic shaders execute according to the chosen API precision and type rules.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse/shape/operation.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/shader/parse/shape/operation.rs)
- [emulator/src/devices/virtio_gpu/three_d/virgl/draw/transform.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/draw/transform.rs)

## Checklist

- [ ] Create child lists for float/vector arithmetic, integers/conversions and comparisons/bit
  operations from the feature matrix.
- [ ] Implement one family at a time with component masks, swizzles, saturation and defined edge
  cases.
- [ ] Differentially compare generated programs against the pinned reference renderer.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Each arithmetic family compares randomized and edge-case outputs using the S01 evaluator against
  the pinned reference with API-appropriate tolerances; browser replay follows S05.
- NaN, infinity, signed zero and integer overflow follow defined API rules or explicit
  undefined-case exclusions.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
