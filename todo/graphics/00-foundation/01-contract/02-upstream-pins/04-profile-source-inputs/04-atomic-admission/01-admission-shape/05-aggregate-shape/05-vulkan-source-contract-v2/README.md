# F02.4.4.1.5.5 — Define the Vulkan source-contract V2

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.4.4.1.5.5
Depends: F02.2, F02.4.3, F02.4.4.1.3, F02.4.4.1.5.1, F03.1
Evidence: pending

Prerequisite lists: the [V1 aggregate](../README.md), [Vulkan audit](../../../../03-vulkan-input-audit/README.md),
and [F03.1](../../../../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

A versioned contract admits the immutable Khronos `vk-default.txt` root as a canonical upstream suite,
not an invented core-only selector. It preserves the 8 MiB regular-source policy, keeps V1 evidence
read-only, and gives F03 a truthful source-availability path without claiming compatibility.

## Starting points

- [F02 regular-source policy](../../../../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [VCTS V1 blocker](../03-vcts-core-closure/evidence.md)
- [Docs V1 blocker](../02-vulkan-docs-closure/evidence.md)
- [F03 source requirements](../../../../../../03-feature-matrix/01-profile-scope/source_requirements.json)

## Checklist

- [x] [F02.4.4.1.5.5.1 — Record the V2 policy decision](01-policy-decision/README.md)
- [x] [F02.4.4.1.5.5.2 — Define the canonical-suite schema](02-canonical-suite-schema/README.md)
- [x] [F02.4.4.1.5.5.3 — Verify the external closure cache](03-external-closure-cache/README.md)
- [x] [F02.4.4.1.5.5.4 — Freeze coverage-report taxonomy](04-coverage-taxonomy/README.md)
- [ ] [F02.4.4.1.5.5.5 — Hand off the V2 aggregate](05-aggregate-handoff/README.md)

## Verification

- V2 never modifies a V1 blocker, filters an official selector, or calls local test coverage Khronos
  conformance.
- A later cutover requires a pinned root, closure ledger, external-cache receipt, and explicit
  `core`/WSI/video/extension/unknown reporting boundary.
