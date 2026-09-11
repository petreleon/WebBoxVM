# F02.5.2.2 — Build a bounded local Vulkan 1.4 definition

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.2.2
Depends: F02.5.1, F02.5.2.1
Evidence: pending

## Outcome

The pinned Vulkan-Docs root and `vk.xml` reproducibly produce a small WebBoxVM facts artifact.
It distinguishes XML `VK_VERSION_1_4` declarations from normative-prose locators and extension
boundaries, and names WebBoxVM—not Khronos—as its producer.

## Starting points

- [normative root records](../01-normative-root-pins/normative_roots.py)
- [shared role contract](../../01-authority-and-transform-boundary/source_role_contract.py)
- [pinned Vulkan build instructions](https://github.com/KhronosGroup/Vulkan-Docs/blob/main/BUILD.adoc)

## Checklist

- [ ] Define an exact no-network builder argv, tool identity, inputs, output digest, and source locators.
- [ ] Extract only documented `VK_VERSION_1_4` XML facts; preserve extension facts as separate boundaries.
- [ ] Rebuild from verified inputs and reject an altered builder, input, output, or claimed selector role.
- [ ] Keep the local output at or below 8 MiB and make no compatibility or performance claim.

## Verification

- Rebuilding the artifact either matches its record byte-for-byte or fails closed.
- The artifact is an engineering definition, not a Khronos selector, profile, implementation claim, or
  substitute for the full CTS root.
