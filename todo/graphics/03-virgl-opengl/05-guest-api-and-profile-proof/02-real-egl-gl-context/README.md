# G14 — Create and exercise a real guest Mesa EGL/OpenGL context

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: G14
Depends: G03, G06, G07, G08, G09, G10, G11, G12, G13
Evidence: pending

Prerequisite lists: [G03](../../01-mesa-ingress/03-screen-capability-queries/README.md), [G06](../../02-resource-and-shader-bindings/03-general-shader-objects/README.md), [G07](../../03-draw-state/01-vertex-and-index-fetch/README.md), [G08](../../03-draw-state/02-framebuffer-depth-stencil/README.md), [G09](../../03-draw-state/03-raster-and-clip-state/README.md), [G10](../../04-pixel-and-query-operations/01-blend-and-channel-state/README.md), [G11](../../04-pixel-and-query-operations/02-sampler-and-texture-state/README.md), [G12](../../04-pixel-and-query-operations/03-copy-blit-and-readback/README.md), [G13](../01-query-results/README.md).

## Outcome

A stock Mesa guest creates and exercises a real EGL/OpenGL context for an explicitly tested earlier
bring-up profile; final F03 profiles remain gated by G15.

## Starting points

- [guest/virgl-clear-demo/README.md](../../../../../guest/virgl-clear-demo/README.md)
- [scripts/virgl_guest_transport_smoke.sh](../../../../../scripts/virgl_guest_transport_smoke.sh)
- [web/js/gpu-display-diagnostics.js](../../../../../web/js/gpu-display-diagnostics.js)
- [web/js/vm-worker/gpu-3d.js](../../../../../web/js/vm-worker/gpu-3d.js)

## Checklist

- [ ] Select a precise earlier bring-up profile whose required semantics are implemented. Test
  larger candidate profiles only in a clearly experimental validation mode; do not present them as
  completed public support.
- [ ] Add a reproducible guest EGL context probe that logs vendor/renderer/version/extensions,
  compiles shaders, renders multiple frames, and checks readback.
- [ ] Correlate guest driver selection and VirGL commands with browser WebGPU execution; detect
  llvmpipe/softpipe or another unintended fallback as a failed accelerated-path result.
- [ ] Exercise resize, off-screen rendering, texture/buffer updates, and context teardown; preserve
  exact first-failure diagnostics and artifacts when startup fails.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- The browser run produces a successful real EGL/GL context report, independently checked pixels,
  multiple presented frames, and evidence that the expected Mesa VirGL driver and WebGPU adapter
  executed the work.
- Run scripts/virgl_guest_transport_smoke.sh with the recorded disk and demo paths for the legacy
  Linux path; report a timeout or missing final PASS as incomplete, never as a pass.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
