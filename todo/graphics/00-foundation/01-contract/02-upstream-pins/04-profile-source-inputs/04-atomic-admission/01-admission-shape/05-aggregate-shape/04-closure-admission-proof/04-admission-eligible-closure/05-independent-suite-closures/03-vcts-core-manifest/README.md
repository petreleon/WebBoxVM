# F02.4.4.1.5.4.4.5.3 — Establish the VCTS core-manifest condition

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.3
Depends: F02.4.3, F02.4.4.1.5.5, F02.4.4.1.5.4.4.5.1
Evidence: pending

Prerequisite lists: the [successor boundary](../01-successor-boundary/README.md), the
[VCTS canonical-suite taxonomy](../../../../05-vulkan-source-contract-v2/04-coverage-taxonomy/README.md), and
[V2 handoff](../../../../05-vulkan-source-contract-v2/05-aggregate-handoff/README.md).

## Outcome

Establish a future condition for an upstream Khronos-published, immutable Vulkan 1.4 core manifest.
The complete `vk-default` root remains a broader canonical diagnostic; neither taxonomy nor local filtering
can turn it into Khronos core conformance evidence.

## Starting points

- [canonical VCTS handoff](../../../../05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff.json)
- [coverage taxonomy](../../../../05-vulkan-source-contract-v2/04-coverage-taxonomy/coverage_taxonomy.json)
- [historical VCTS blocker](../../../../03-vcts-core-closure/evidence.md)
- [successor boundary](../01-successor-boundary/README.md)

## Checklist

- [x] [Recheck the official upstream selector and retain its blocker](01-upstream-selector-recheck/README.md)
- [ ] [Validate an authoritative explicit-core manifest when Khronos publishes one](02-validate-authoritative-core-manifest/README.md)
- [ ] [Capture and replay the future core closure](03-capture-and-replay-core-closure/README.md)
- [ ] [Reconcile the future VCTS condition atomically](04-reconcile-vcts-condition/README.md)

## Verification

- `satisfies_vulkan_14_core_manifest` remains false unless the new condition and atomic transition both pass.
- This task does not claim Vulkan CTS execution, Khronos certification, guest API support, browser behavior, or performance.
