# S05 — Emit and validate WGSL from shared IR

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: S05
Depends: S01
Evidence: pending

Prerequisite lists: [S01](../../01-frontends/01-typed-ir/README.md).

## Outcome

Shader generation is reusable and checked before a guest draw can report success.

## Starting points

- [web/js/webgpu-virgl-material-batch-shaders.js](../../../../../web/js/webgpu-virgl-material-batch-shaders.js)
- [web/js/webgpu-errors.js](../../../../../web/js/webgpu-errors.js)

## Checklist

- [ ] Emit WGSL expressions, structured control flow and binding layouts from validated IR.
- [ ] Capture compilation diagnostics and asynchronous pipeline validation errors with guest shader
  IDs.
- [ ] Split emitter modules by responsibility and keep generated files below 180 lines.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Real browser compilation and execution succeed for the initial IR corpus on the recorded adapter.
- Invalid WGSL cannot escape as an uncaught promise or successful guest completion.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
