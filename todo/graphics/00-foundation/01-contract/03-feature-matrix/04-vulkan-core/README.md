# F03.4 — Import the Vulkan 1.4 core inventory

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.4
Depends: F03.1, F02.5.4.2
Evidence: pending

Prerequisite lists: [F03.1](../01-profile-scope/README.md) and [the active role-aware gate]
(../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md).

## Outcome

Every mandatory Vulkan 1.4 core command, feature, limit, format, synchronization rule, and shader
requirement has a stable source locator, an implementation owner, and a reference-test obligation.

## Starting points

- [sealed role-aware source contract](../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)
- [active F03 role-aware gate](../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md)
- [raw registry inventory](01-registry-inventory/README.md)
- [Venus feasibility evidence](../../../../../../research/venus-foundations.md)
- [profile schema](../01-profile-scope/README.md)
- [active role-aware gate](../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md)

## Checklist

- [x] [F03.4.1 — Enumerate the Vulkan registry inventory](01-registry-inventory/README.md)
- [x] [F03.4.2 — Attach Docs provenance and CTS diagnostics](02-provenance-diagnostics/README.md)
- [ ] [F03.4.3 — Review semantic core scope, owners, and test plans](03-semantic-core-owner-and-test-plan/README.md)

## Verification

- No mandatory Vulkan 1.4 core row lacks a source locator, owner, or reference-test plan.
- A stale registry/grammar/CTS identity, missing row, duplicate row, or unproven supported row fails
  the focused check. The split preserves that vk.xml is not a normative-spec or CTS-selector substitute.

## Split rationale

The registry can enumerate technical API facts after the shared source gate, whereas raw Docs citations
establish normative provenance and the canonical CTS suite supplies only broad diagnostics.
Keeping these tasks separate prevents a registry fragment, generated Docs, or local CTS filtering from
silently claiming the complete Vulkan 1.4 core inventory.

F03.4.2 now separates source-channel boundaries, raw Docs citations, full-suite diagnostics, and their
aggregate receipt. F03.4.3 can then review a semantic public-core scope and assign ownership/test plans
without relabeling the 1,458 raw registry facts as an already-complete matrix.
