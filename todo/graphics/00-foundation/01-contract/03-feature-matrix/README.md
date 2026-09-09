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

- [ ] [F03.1 — Define the profile scope and matrix schema](01-profile-scope/README.md)
- [ ] [F03.2 — Import the OpenGL 4.6 core inventory](02-opengl-core/README.md)
- [ ] [F03.3 — Import the GLES 3.2 inventory](03-gles/README.md)
- [ ] [F03.4 — Import the Vulkan 1.4 core inventory](04-vulkan-core/README.md)
- [ ] Run the named F05 coverage check, review every blocked row, and attach the F03 aggregate receipt.

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

## Split rationale

OpenGL core, GLES, and Vulkan have separate normative inputs, feature taxonomies, implementation
lanes, and reference suites. The shared profile schema and source-sufficiency gate are deliberately
separated from those imports; the parent verifies cross-profile ownership and test coverage only after
F05 supplies the named check runner. This keeps a future blocked mandatory requirement visible instead
of silently shrinking the final target.
