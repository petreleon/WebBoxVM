# I06 — Verify browsers, workers and interactive presentation

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: I06
Depends: I02, I03, R08
Evidence: pending

Prerequisite lists: [I02](../../01-real-clients/02-mesa-opengl/README.md), [I03](../../01-real-clients/03-mesa-vulkan/README.md), [R08](../../../01-shared-runtime/02-execution/04-device-loss/README.md).

## Outcome

Guest output and input work across a declared browser/OS/GPU matrix.

## Starting points

- [Makefile](../../../../../Makefile)
- [scripts/serve_web.py](../../../../../scripts/serve_web.py)
- [web/js/vm-worker/wasm.js](../../../../../web/js/vm-worker/wasm.js)
- [web/js/gpu-display.js](../../../../../web/js/gpu-display.js)

## Checklist

- [ ] Pin supported browser/OS/adapter combinations and test fresh serial and threaded Wasm builds.
- [ ] Exercise resize, scale factor, hidden-tab recovery, input, device loss and browser resource
  limits.
- [ ] Keep screenshots/readback and API-reported renderer identity with every run; log unsupported
  platform rows.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run make web-pkg and execute the documented real-browser smoke on every required matrix row.
- No stale cache, software fallback or mocked GPU result can satisfy browser acceptance.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
