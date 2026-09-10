# F02.4.4.1.5.4.4 — Establish an admission-eligible closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4
Depends: F02.4.4.1.5.4.3
Evidence: pending

Prerequisite lists: the [blocked-state receipt](../03-blocked-state-receipt/README.md),
[V2 source contract](../../05-vulkan-source-contract-v2/README.md), and [F02.2 policy](../../../../../../02-fetch-verifier/README.md).

## Outcome

One later active, complete, policy-valid successor closure may turn the six-role aggregate
admission-eligible without substituting V2 VCTS for Docs or weakening the immutable scope rules.
It still cannot perform inventory cutover, prove cache freshness, or alter F03.

## Starting points

- [blocked-state receipt](../03-blocked-state-receipt/README.md)
- [Docs provenance boundary](../../05-vulkan-source-contract-v2/05-aggregate-handoff/README.md)
- [source requirements](../../../../../../../03-feature-matrix/01-profile-scope/source_requirements.json)

## Checklist

- [ ] Establish an explicit active successor policy and complete immutable closure for `vulkan-14-spec`.
- [ ] Reconcile all six roles through their native V1 or V2 envelopes without aliases, root-only substitutes, or scope erosion.
- [ ] Prove the real result `admission_eligible=true` while retaining no inventory, fresh-cache, cutover, or F03 claim.
- [ ] Reject a Docs provenance-only record, VCTS-as-Docs substitution, stale evidence, and false-ready promotion.
- [ ] Run focused positive and hostile tests and attach exact PASS evidence.

## Verification

- Current state: **BLOCKED** — V2 verifies an external Khronos suite but preserves `admitted=false`,
  `cutover_ready=false`, and `satisfies_vulkan_14_core_manifest=false`; Docs remains provenance, not an
  admitted implementation source. This task stays unchecked until that separate source truth changes.
- `admission_closures.json` and every downstream cutover/F03 effect remain reserved for F02.4.4.2–.5.
