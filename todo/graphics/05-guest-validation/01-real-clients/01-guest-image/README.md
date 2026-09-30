# I01 — Build a reproducible graphics guest image

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: I01
Depends: F02, F05
Evidence: partial — [image automation and real guest receipt](automation-evidence.md)

Prerequisite lists: [F02](../../../00-foundation/01-contract/02-upstream-pins/README.md), [F05](../../../00-foundation/02-reproducibility/02-check-runner/README.md).

## Outcome

The guest has pinned unmodified Mesa drivers and tools that can exercise standard APIs.

## Starting points

- [guest/virgl-clear-demo/Makefile](../../../../../guest/virgl-clear-demo/Makefile)
- [scripts/virgl_guest_transport_smoke.sh](../../../../../scripts/virgl_guest_transport_smoke.sh)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Create a reproducible ARM64 image recipe containing kernel virtio-gpu, Mesa VirGL/Venus,
  EGL/GL and Vulkan tools.
- [x] Record package/source hashes, build flags and driver selection; keep installed disk data
  outside Git.
- [x] Add a boot/command runner that captures serial output and exits on timeout or guest failure.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

The [input capture receipt](input-capture-evidence.md) verifies the exact full Mesa archive and
its F02 VirGL/Venus source hashes. The [automation receipt](automation-evidence.md)
records a built stock-Mesa image, successful kernel/GPU/tool command execution,
and actual GLES/Vulkan startup failures. Clean Mesa binary reproduction and
successful renderer/ICD reporting remain unverified; I01 stays open.

- A clean image build reproduces the manifest and guest tool versions.
- Renderer/ICD reporting identifies software fallback separately; missing drivers are a failure.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
