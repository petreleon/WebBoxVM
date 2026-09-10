# F03.4.1 — Enumerate the Vulkan registry inventory

[Parent task](../README.md)

Task: F03.4.1
Depends: F03.1, F02.4.4.1.5.4.4.3
Evidence: [receipt](evidence.md)

Prerequisite lists: [the profile schema](../../01-profile-scope/README.md) and [the source-release
boundary](../../../02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/04-closure-admission-proof/04-admission-eligible-closure/03-source-release-boundary/README.md).

## Outcome

One blocked technical inventory enumerates direct Vulkan 1.0–1.4 commands, types, enums, features, and
version markers from the exact pinned registry. It uses the source-ordered base, compute, graphics, and
public blocks, retaining `require`/`deprecate` membership, exact locators, and unassigned owner/test-plan
placeholders. The internal blocks are structural lookup material, not a semantic public-core closure.

## Starting points

- [profile schema](../../01-profile-scope/README.md)
- [source-release boundary](../../../02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/04-closure-admission-proof/04-admission-eligible-closure/03-source-release-boundary/README.md)

## Checklist

- [x] Pin the exact registry identity, size, SPDX expression, and Vulkan 1.4 version marker.
- [x] Extract only stable registry facts into a separate blocked technical schema with exact source locators.
- [x] Record explicit extension, WSI, external-memory, SPIR-V, and bring-up boundaries.
- [x] Require every row to remain blocked with null owner and independent-test-plan placeholders.
- [x] Reject stale, duplicate, partial, malformed, or supported-by-inference rows with focused tests and a receipt.

## Verification

- A registry inventory cannot satisfy the normative Docs role, select CTS cases, advertise Venus/Vulkan
  support, or complete F03.

## Boundary

The emitted 1,458 rows are raw structural facts only. They include 20 version markers, 260 commands,
668 types, 393 enums, and 117 feature references; 88 are direct `deprecate` members. Extension blocks,
WSI, and external-memory extension/platform-handle paths are excluded, while core-promoted structural rows
remain recorded. The retained
`vk.xml` identity and byte-for-byte row digest reject a changed, partial, reordered, or locally promoted
inventory. This artifact intentionally is not a `matrix_contract.py` matrix: the future normative Docs
and must-pass source IDs are still unavailable, and the source/release boundary forbids substitution.
