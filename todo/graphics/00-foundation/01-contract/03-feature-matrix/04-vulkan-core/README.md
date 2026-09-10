# F03.4 — Import the Vulkan 1.4 core inventory

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.4
Depends: F02, F03.5
Evidence: pending

Prerequisite lists: [F02](../../02-upstream-pins/README.md) and
[F03.5](../05-source-contract-v2/README.md).

## Outcome

Every mandatory Vulkan 1.4 core command, feature, limit, format, synchronization rule, and shader
requirement has a stable source locator, an implementation owner, and a reference-test obligation.

## Starting points

- [Vulkan registry input](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [SPIR-V grammar input](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [Vulkan CTS input](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [Venus feasibility evidence](../../../../../../research/venus-foundations.md)
- [profile schema](../01-profile-scope/README.md)
- [source-contract v2](../05-source-contract-v2/README.md)

## Checklist

- [x] [F03.4.1 — Enumerate the Vulkan registry inventory](01-registry-inventory/README.md)
- [ ] [F03.4.2 — Attach Docs provenance and CTS diagnostics](02-provenance-diagnostics/README.md)

## Verification

- No mandatory Vulkan 1.4 core row lacks a source locator, owner, or reference-test plan.
- A stale registry/grammar/CTS identity, missing row, duplicate row, or unproven supported row fails
  the focused check. The split preserves that vk.xml is not a normative-spec or CTS-selector substitute.

## Split rationale

The registry can enumerate technical API facts before a full source-contract cutover, whereas raw Docs
citations establish normative provenance and the canonical CTS suite supplies only broad diagnostics.
Keeping these tasks separate prevents a registry fragment, generated Docs, or local CTS filtering from
silently claiming the complete Vulkan 1.4 core inventory.
