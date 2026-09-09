# I05 — Validate real application behavior

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: I05
Depends: I02, I03, I04
Evidence: pending

Prerequisite lists: [I02](../../01-real-clients/02-mesa-opengl/README.md), [I03](../../01-real-clients/03-mesa-vulkan/README.md), [I04](../01-conformance-runner/README.md).

## Outcome

Compatibility works for ordinary guest programs and interaction, not only synthetic draws.

## Starting points

- [guest/virgl-clear-demo/README.md](../../../../../guest/virgl-clear-demo/README.md)
- [web/js/gpu-display.js](../../../../../web/js/gpu-display.js)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Pin at least one multi-shader GL app, one Vulkan app and one desktop/compositor workload with
  deterministic scenes.
- [ ] Record launch, resize, animation, input, multi-context use and screenshot/semantic
  checkpoints.
- [ ] Minimize each unsupported command or rendering failure into its owning protocol task; keep app
  failures open.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Each application runs through stock guest APIs on hardware WebGPU with reference-checked output.
- A forced software renderer is detected and cannot satisfy the accelerated application lane.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
