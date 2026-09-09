# V16 — Implement stock guest Vulkan WSI acquire/present and resize

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V16
Depends: V06, V08, V13, R08
Evidence: pending

Prerequisite lists: [V06](../../02-memory-and-interop/03-external-memory-feasibility-proof/README.md), [V08](../../03-command-and-sync-state/02-queue-fences-binary-semaphores/README.md), [V13](../../05-work-and-presentation/01-graphics-draw-execution/README.md), [R08](../../../01-shared-runtime/02-execution/04-device-loss/README.md).

## Outcome

The selected stock Mesa guest window-system integration presents swapchain images through the
browser with correct ownership and completion.

## Starting points

- [guest/virgl-clear-demo/kms.c](../../../../../guest/virgl-clear-demo/kms.c)
- [web/js/gpu-display.js](../../../../../web/js/gpu-display.js)
- [web/js/webgpu-scanout.js](../../../../../web/js/webgpu-scanout.js)
- [web/js/gpu-display-reset.test.mjs](../../../../../web/js/gpu-display-reset.test.mjs)

## Checklist

- [ ] Choose and document the actual guest WSI path supported by the pinned environment, then
  implement its surface formats, swapchain images, and presentation mode semantics.
- [ ] Connect acquire/present semaphore/fence behavior, image ownership, and browser scanout without
  reading every frame through guest CPU memory.
- [ ] Handle resize, out-of-date/suboptimal surfaces, minimize/restore, teardown, and device loss
  with correct guest-visible results and no stranded waits.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- A real guest Vulkan window renders a sustained frame sequence, resizes repeatedly, and recreates
  its swapchain while all acquire/present synchronization remains valid.
- Capture presented pixel checks, swapchain status transitions, pending-object counts, and bytes
  read back per frame; browser presentation must be correlated with the guest's submitted images.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
