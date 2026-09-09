# I03 — Verify a stock Mesa Venus Vulkan guest in a real browser

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: I03
Depends: I01, V16, V17, F05
Evidence: pending

Prerequisite lists: [I01](../01-guest-image/README.md), [V16](../../../04-venus-vulkan/06-stock-guest-and-profile-proof/01-wsi-acquire-present/README.md), [V17](../../../04-venus-vulkan/06-stock-guest-and-profile-proof/02-stock-instance-device-properties/README.md), [F05](../../../00-foundation/02-reproducibility/02-check-runner/README.md).

## Outcome

Unmodified Mesa Venus renders an explicitly named earlier bring-up Vulkan profile through browser
WebGPU; final-profile acceptance remains V18/Q02.

## Starting points

- [guest/virgl-clear-demo/README.md](../../../../../guest/virgl-clear-demo/README.md)
- [scripts/virgl_guest_transport_smoke.sh](../../../../../scripts/virgl_guest_transport_smoke.sh)
- [web/js/gpu-display-diagnostics.js](../../../../../web/js/gpu-display-diagnostics.js)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Run Vulkan device enumeration plus a pinned sample that uses SPIR-V, descriptors,
  synchronization and WSI presentation.
- [ ] Correlate guest API and real Venus wire records with browser dispatch, completed GPU work and
  pixel output.
- [ ] Test resize, readback and device-loss reporting; detect lavapipe, native helper, remote
  rendering or private transport substitution.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Fresh serial and threaded Wasm runs identify the Venus ICD, exact API profile and hardware WebGPU
  adapter.
- Native reference and browser guest images/readback match, and a forced software ICD causes the
  accelerated lane to fail.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
