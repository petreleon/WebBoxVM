# F02.4.4.1.3 — Bound unresolved Vulkan closures

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.4.4.1.3
Depends: F02.1, F02.2, F02.4.3, F02.4.4.1.2
Evidence: pending

Prerequisite lists: [Vulkan audit](../../../03-vulkan-input-audit/README.md) and the required-source map.

## Outcome

The source model records why neither Vulkan root is a closure: generated/transitive specification
material remains unpinned, and the VCTS root lacks independently bounded members and core-only scope.
It keeps WSI, video, extensions, and oversize inputs outside any future core admission.

## Starting points

- [Vulkan include contract](../../../03-vulkan-input-audit/spec_include_contract.py)
- [VCTS references](../../../03-vulkan-input-audit/mustpass_references.json)
- [Vulkan candidate audit](../../../03-vulkan-input-audit/candidates.json)

## Checklist

- [ ] Bind the Vulkan Docs root to its incomplete generated/transitive closure status.
- [ ] Bind the VCTS root to its unpinned members, oversize inputs, and non-core scope status.
- [ ] Record WSI, video, and extension exclusions with no fallback-to-root rule.
- [ ] Add focused hostile checks for fabricated members, missing blockers, and scope expansion.
- [ ] Keep both logical IDs explicitly unadmitted until every member and limit is pinned.

## Verification

- No Vulkan record can become an inventory source, F03 input, execution suite, or API claim.
- A documented blocker is not a completed Vulkan compatibility milestone.
