# S09 — Lower SPIR-V graphics operations into shared IR

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S09
Depends: S08, S05, S06
Evidence: pending

Prerequisite lists: [S08](../01-spirv-validation/README.md), [S05](../../02-backend/01-wgsl-emission/README.md), [S06](../../02-backend/02-stage-interfaces/README.md).

## Outcome

Real Vulkan vertex and fragment shaders execute through the same checked backend.

## Starting points

- [web/js/webgpu-3d.js](../../../../../web/js/webgpu-3d.js)
- [web/js/webgpu-errors.js](../../../../../web/js/webgpu-errors.js)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)

## Checklist

- [ ] Lower typed arithmetic, structured control flow and descriptor-backed loads from validated
  SPIR-V.
- [ ] Implement specialization/layout handling and split missing operation families into child
  lists.
- [ ] Compare identical GLSL-origin programs through the TGSI and SPIR-V routes.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Pinned shader corpus produces matching images through both APIs and native references.
- Unsupported instructions return a specific error and leave mandatory coverage visibly incomplete.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
