# F03.4.1 — Enumerate the Vulkan registry inventory

[Parent task](../README.md)

Task: F03.4.1
Depends: F03.1, F02.4.4.1.5.4.4.3
Evidence: pending

Prerequisite lists: [the profile schema](../../01-profile-scope/README.md) and [the source-release
boundary](../../../02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/04-closure-admission-proof/04-admission-eligible-closure/03-source-release-boundary/README.md).

## Outcome

One blocked technical inventory enumerates Vulkan 1.4 commands, types, features, and version markers from
the pinned registry with exact locators, owner assignments, and test-plan placeholders. It is not a
normative-completeness, implementation, or conformance result.

## Starting points

- [profile schema](../../01-profile-scope/README.md)
- [source-release boundary](../../../02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/04-closure-admission-proof/04-admission-eligible-closure/03-source-release-boundary/README.md)

## Checklist

- [ ] Pin the exact registry identity, size, SPDX expression, and Vulkan 1.4 version marker.
- [ ] Extract only stable registry facts into the shared row schema with exact source locators.
- [ ] Record explicit extension, WSI, external-memory, SPIR-V, and bring-up boundaries.
- [ ] Require each row to remain blocked or unimplemented until an owner and independent test plan exist.
- [ ] Reject stale, duplicate, partial, or supported-by-inference rows with focused tests and a receipt.

## Verification

- A registry inventory cannot satisfy the normative Docs role, select CTS cases, advertise Venus/Vulkan
  support, or complete F03.
