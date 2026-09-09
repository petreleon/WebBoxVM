# V17 — Expose accurate Venus/Vulkan properties and create a stock Mesa device

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V17
Depends: R01, V03, V06, V11, V13, V14, V15
Evidence: pending

Prerequisite lists: [R01](../../../01-shared-runtime/01-resources/01-capability-registry/README.md), [V03](../../01-real-protocol/03-rings-replies-negotiation/README.md), [V06](../../02-memory-and-interop/03-external-memory-feasibility-proof/README.md), [V11](../../04-pipeline-and-execution/02-shader-and-pipeline-creation/README.md), [V13](../../05-work-and-presentation/01-graphics-draw-execution/README.md), [V14](../../05-work-and-presentation/02-compute-dispatch-execution/README.md), [V15](../../05-work-and-presentation/03-transfer-and-query-commands/README.md).

## Outcome

Stock Mesa Venus creates a usable Vulkan device for an explicitly tested earlier bring-up profile;
final F03 properties and claims remain gated by V18.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/capset.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/capset.rs)
- [research/venus-foundations.md](../../../../../research/venus-foundations.md)
- [web/js/gpu-display-diagnostics.js](../../../../../web/js/gpu-display-diagnostics.js)
- [scripts/virgl_guest_transport_smoke.sh](../../../../../scripts/virgl_guest_transport_smoke.sh)

## Checklist

- [ ] Choose a precise bring-up profile and query only its implemented properties. Permit explicitly
  experimental candidate-profile CTS runs to find gaps before public claims; record every mandatory
  failure.
- [ ] Generate capset 4 negotiation and Vulkan properties/features/extensions/limits from the shared
  implementation registry, including required pNext query structures.
- [ ] Build and run a reproducible stock guest instance/device probe and vulkaninfo-equivalent
  report, recording ICD selection and excluding lavapipe or unintended fallback.
- [ ] Keep the public capability disabled until all mandatory initialization prerequisites and
  external-memory requirements pass; preserve the first real protocol failure otherwise.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- The guest report identifies the expected Venus ICD, successful instance/device creation, exact API
  version, queue families, memory properties, limits, and extension/feature lists matching browser
  support.
- Every advertised property has a registry implementation/test reference; stock-Mesa initialization
  and one real graphics plus compute result pass through the browser before an accelerated Vulkan
  claim.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
