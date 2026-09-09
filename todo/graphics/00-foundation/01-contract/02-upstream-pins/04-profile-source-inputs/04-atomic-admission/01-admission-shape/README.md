# F02.4.4.1 — Define closure-aware admission shape

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.4.4.1
Depends: F02.1, F02.2, F02.4.1, F02.4.2, F02.4.3
Evidence: pending

Prerequisite lists: the three [source audits](../README.md) and
[F02.2](../../../02-fetch-verifier/README.md).

## Outcome

A fail-closed source model distinguishes a pinned root from an admissible logical closure. It defines
pre-admission candidate semantics and post-admission inventory semantics without allowing a rejected
GLES or Vulkan root to masquerade as a complete F02.2 source.

## Starting points

- [candidate contract](../../candidate_contract.py)
- [GLES compound contract](../../compound_selector_contract.py)
- [Vulkan rejected closures](../../03-vulkan-input-audit/README.md)
- [F02 source policy](../../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [F03 source gate](../../../../03-feature-matrix/01-profile-scope/profile_contract.py)

## Checklist

- [ ] [F02.4.4.1.1 — Probe the GLES rejected closure](01-gles-closure-probe/README.md)
- [ ] [F02.4.4.1.2 — Map all required-source shapes](02-source-map/README.md)
- [ ] [F02.4.4.1.3 — Bound unresolved Vulkan closures](03-vulkan-boundaries/README.md)
- [ ] [F02.4.4.1.4 — Define post-cutover rules](04-post-cutover-rules/README.md)
- [ ] [F02.4.4.1.5 — Aggregate the admission shape](05-aggregate-shape/README.md)

## Verification

- A root alone never satisfies a compound source requirement, even if its URL, revision, and digest match.
- The model preserves F02.2's byte, cache, family, and immutable-source rules without claiming support.

## Split rationale

The present GLES audit can support a small pre-admission probe, but it cannot establish the all-source
mapping or post-cutover behavior while Vulkan generated/transitive and core-scope closure is unresolved.
These children keep that proven GLES boundary independent from the source map, Vulkan blockers, and a
future inventory transition model. No child may reclassify a rejected root or alter the inventory alone.
