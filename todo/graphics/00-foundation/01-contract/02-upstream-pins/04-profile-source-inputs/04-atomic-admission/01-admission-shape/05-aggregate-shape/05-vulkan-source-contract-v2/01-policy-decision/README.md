# F02.4.4.1.5.5.1 — Record the V2 policy decision

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.5.1
Depends: F02.2, F02.4.3, F02.4.4.1.3, F02.4.4.1.5.1, F03.1
Evidence: pending

Prerequisite lists: the [V1 VCTS blocker](../../03-vcts-core-closure/evidence.md), the
[V1 Docs blocker](../../02-vulkan-docs-closure/evidence.md), and [F03.1](../../../../../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

An explicit, reproducible decision records why V1 is superseded and what V2 may prove. It preserves
Vulkan 1.4 core as the product target while separating local core readiness, full CTS diagnostics,
and official Khronos conformance.

## Starting points

- [replacement goal](../../../../../../../../../goal.md)
- [V1 aggregate](../../README.md)
- [VCTS source audit](../../../../../03-vulkan-input-audit/README.md)

## Checklist

- [ ] Record the user-approved V2 boundary and the exact V1 blockers it replaces.
- [ ] Define `regular-source`, `canonical-upstream-suite`, and non-distributable generated Docs roles.
- [ ] State that local core coverage and a complete `vk-default` run are separate evidence levels.
- [ ] Add checker coverage proving superseded is not PASS and cannot unblock an active dependency alone.
- [ ] Run focused decision/checker checks and attach the bounded evidence receipt.

## Verification

- No claim changes from blocked to supported, conformant, certified, or near-native in this decision.
- The decision fails if it omits either the immutable-root requirement or the no-local-selector rule.
