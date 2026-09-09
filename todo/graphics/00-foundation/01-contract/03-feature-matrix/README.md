# F03 — Freeze API profiles and their complete feature inventory

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: F03
Depends: F02
Evidence: pending

Prerequisite lists: [F02](../02-upstream-pins/README.md).

## Outcome

Completion has a versioned, enumerated meaning for OpenGL, GLES and Vulkan.

## Starting points

- [research/virgl-compatibility.md](../../../../../research/virgl-compatibility.md)
- [research/virgl2-capset.md](../../../../../research/virgl2-capset.md)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)

## Checklist

- [ ] Create small per-API matrices: draft final targets OpenGL 4.6 core, GLES 3.2 and Vulkan 1.4
  core; record compatibility-profile and extension scope explicitly.
- [ ] Import every mandatory command, feature, limit, format and shader requirement from the pinned
  registries, linking each to a task and reference test.
- [ ] Use earlier API versions for bring-up only; tag supported, emulated, unsupported and blocked
  rows with evidence, never silently lower the final target.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- The matrix has no unassigned mandatory row and no supported row without a test reference.
- Any impossible mandatory feature keeps its profile incomplete; changes of final scope require an
  explicit recorded user decision.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
