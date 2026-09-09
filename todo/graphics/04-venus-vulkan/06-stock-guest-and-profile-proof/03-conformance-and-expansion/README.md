# V18 — Close the Vulkan profile and every requested expansion gap

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V18
Depends: F03, F05, V16, V17, I03, I04
Evidence: pending

Prerequisite lists: [F03](../../../00-foundation/01-contract/03-feature-matrix/README.md), [F05](../../../00-foundation/02-reproducibility/02-check-runner/README.md), [V16](../01-wsi-acquire-present/README.md), [V17](../02-stock-instance-device-properties/README.md), [I03](../../../05-guest-validation/01-real-clients/03-mesa-vulkan/README.md), [I04](../../../05-guest-validation/02-independent-validation/01-conformance-runner/README.md).

## Outcome

A reproducible Vulkan CTS report accounts for every mandatory requirement and keeps all remaining
compatibility work in explicit small child lists.

## Starting points

- [research/venus-foundations.md](../../../../../research/venus-foundations.md)
- [research/virgl2-capset.md](../../../../../research/virgl2-capset.md)
- [emulator/src/devices/virtio_gpu/three_d/capset.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/capset.rs)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Pin Vulkan CTS revisions and must-pass selections for the exact Vulkan
  version/profile/extensions; record valid expected limitations and browser prerequisites.
- [ ] Run conformance with the stock guest ICD and expand each mandatory failure into a bounded
  child task, including events, advanced stages, sparse/resource aliasing, additional queue
  semantics, extension features, and portability differences where required.
- [ ] Retain unresolved broader compatibility targets as open goal requirements; do not redefine
  full compatibility or near-native success as the minimum subset that currently renders.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Raw CTS logs, reproducible commands, guest/browser/GPU versions, case counts, and a complete
  requirements matrix contain no unexplained mandatory failure or skip for a claimed profile.
- Run make test and make web-pkg for release candidates; capset exposure, Vulkan compatibility
  claims, and goal completion require both conformance and real guest-browser application evidence.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
