# I02 — Verify a stock Mesa EGL/OpenGL guest in a real browser

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: I02
Depends: I01, G14, F05
Evidence: pending

Prerequisite lists: [I01](../01-guest-image/README.md), [G14](../../../03-virgl-opengl/05-guest-api-and-profile-proof/02-real-egl-gl-context/README.md), [F05](../../../00-foundation/02-reproducibility/02-check-runner/README.md).

## Outcome

Unmodified Mesa VirGL renders an explicitly named earlier bring-up profile in the browser;
final-profile acceptance remains G15/Q02.

## Starting points

- [guest/virgl-clear-demo/README.md](../../../../../guest/virgl-clear-demo/README.md)
- [scripts/virgl_guest_transport_smoke.sh](../../../../../scripts/virgl_guest_transport_smoke.sh)
- [web/js/gpu-display-diagnostics.js](../../../../../web/js/gpu-display-diagnostics.js)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Run a pinned EGL/OpenGL program that reports API/renderer, compiles non-template shaders and
  presents multiple frames.
- [ ] Correlate its API calls with VirGL transport, browser GPU execution, completion and verified
  pixels.
- [ ] Test readback, resize and failure reporting; reject llvmpipe/softpipe or private-capset paths
  as accelerated proof.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Fresh serial and threaded Wasm runs report the selected Mesa driver and hardware adapter, with
  zero unexpected GPU errors.
- Image probes and guest glReadPixels match the reference; guest PASS or a native self-acknowledged
  packet alone cannot pass.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
