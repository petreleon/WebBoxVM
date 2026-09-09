# F03.4 — Import the Vulkan 1.4 core inventory

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.4
Depends: F02, F03.1
Evidence: pending

Prerequisite lists: [F02](../../02-upstream-pins/README.md) and
[F03.1](../01-profile-scope/README.md).

## Outcome

Every mandatory Vulkan 1.4 core command, feature, limit, format, synchronization rule, and shader
requirement has a stable source locator, an implementation owner, and a reference-test obligation.

## Starting points

- [Vulkan registry input](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [SPIR-V grammar input](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [Vulkan CTS input](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [Venus feasibility evidence](../../../../../../research/venus-foundations.md)
- [profile schema](../01-profile-scope/README.md)

## Checklist

- [ ] Extract Vulkan 1.4 core commands, features, limits, formats, synchronization, and SPIR-V rules
  from the pinned inputs into the shared row schema with exact source locators.
- [ ] Record the core profile, adopted extension policy, required WSI and external-memory boundaries,
  and bring-up versions explicitly; do not treat a host-only blob profile as Venus support.
- [ ] Link every mandatory row to a downstream implementation task and independent native/guest or
  conformance reference test; retain missing mappings as unassigned or blocked.
- [ ] Classify current browser and Venus feasibility evidence without inventing capset 4, Vulkan
  properties, synchronization, external-memory, or device support.
- [ ] Keep this leaf blocked if F03.1 finds a required authoritative source or complete CTS manifest
  missing; do not infer one from a registry fragment or isolated API-version file.
- [ ] Add focused extraction/coverage/negative tests, run required gates, and attach a receipt.

## Verification

- No mandatory Vulkan 1.4 core row lacks a source locator, owner, or reference-test plan.
- A stale registry/grammar/CTS identity, missing row, duplicate row, or unproven supported row fails
  the focused check.
