# F02.4.4.1.5.4.4.3 — Separate source sufficiency from release conformance

[Parent task](../README.md)

Task: F02.4.4.1.5.4.4.3
Depends: F02.4.4.1.5.4.4.1, F02.4.4.1.5.5
Evidence: pending

Prerequisite lists: the [derived-Docs policy](../01-derived-docs-policy/README.md), [V2 handoff](../../../05-vulkan-source-contract-v2/05-aggregate-handoff/README.md), and the [F03 gate](../../../../../05-f03-gate/README.md).

## Outcome

One versioned boundary separates source-sufficiency evidence for a future feature matrix from actual
guest compatibility, CTS conformance, Khronos certification, and browser-performance release claims.
It preserves the complete `vk-default` suite and explicit Vulkan 1.4 core target without inventing a
local core CTS selector.

## Starting points

- [F03 source-contract target](../../../../../../../../03-feature-matrix/05-source-contract-v2/README.md)
- [V2 coverage taxonomy](../../../05-vulkan-source-contract-v2/04-coverage-taxonomy/README.md)
- [final goal](../../../../../../../../../../goal.md)

## Checklist

- [ ] Define source-sufficiency states that cannot imply implementation support, conformance, certification, or performance.
- [ ] Keep `vk-default` immutable and report core, WSI, video, extension, and unknown categories without filtering it.
- [ ] Require a later source-contract version to bind every profile role and its exact closure identity.
- [ ] Reserve release claims for independent guest, native-reference, CTS, browser, and performance evidence.
- [ ] Reject a local selector, category report, Docs output, or false flag as a release/conformance substitute.
- [ ] Add bounded positive and hostile tests plus a policy receipt without changing F03 or inventory state.

## Verification

- This boundary is a truthfulness control, not a route around incomplete required source closures.
- Until a later atomic transition passes, F03 remains blocked and no API profile is advertised.
