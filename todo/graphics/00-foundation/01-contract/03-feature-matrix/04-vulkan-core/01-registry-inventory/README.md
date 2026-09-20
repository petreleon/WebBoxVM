# F03.4.1 — Enumerate the Vulkan registry inventory

[Parent task](../README.md)

Task: F03.4.1
Depends: F03.1, F02.5.4.2
Evidence: [v2 receipt](evidence.md)

Prerequisite lists: [the profile schema](../../01-profile-scope/README.md) and [the active role-aware
gate](../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md).

## Outcome

One blocked technical inventory enumerates direct Vulkan 1.0–1.4 commands, types, enums, features, and
version markers from the exact pinned registry. It uses the source-ordered base, compute, graphics, and
public blocks, retaining `require`/`deprecate` membership, exact locators, and unassigned owner/test-plan
placeholders. The internal blocks are structural lookup material, not a semantic public-core closure.

## Starting points

- [profile schema](../../01-profile-scope/README.md)
- [active role-aware gate](../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md)

## Checklist

- [x] Revalidate the exact registry identity, size, SPDX expression, and Vulkan 1.4 version marker.
- [x] Revalidate the stable technical schema and exact source locators against F02.5.4.2.
- [x] Reconfirm extension, WSI, external-memory, SPIR-V, and bring-up boundaries.
- [x] Keep every row blocked with null owner and independent-test-plan placeholders.
- [x] Re-run stale, duplicate, partial, malformed, and inference-negative tests and attach a receipt.

## Verification

- A registry inventory cannot satisfy the normative Docs role, select CTS cases, advertise Venus/Vulkan
  support, or complete F03.

## Boundary

The emitted 1,458 rows are raw structural facts only. They include 20 version markers, 260 commands,
668 types, 393 enums, and 117 feature references; 88 are direct `deprecate` members. Extension blocks,
WSI, and external-memory extension/platform-handle paths are excluded, while core-promoted structural rows
remain recorded. The retained
`vk.xml` identity and byte-for-byte row digest reject a changed, partial, reordered, or locally promoted
inventory. This artifact intentionally is not a `matrix_contract.py` matrix: the prior
[v1 receipt](evidence-v1.md) is historical. The active v2 reader loads F02.5.4.2 only through its sealed
role-aware binding, accepts `vulkan-registry` only as `registry-metadata`, verifies a complete fresh
external selector cache, and still cannot discharge a normative Docs role or a CTS role.
