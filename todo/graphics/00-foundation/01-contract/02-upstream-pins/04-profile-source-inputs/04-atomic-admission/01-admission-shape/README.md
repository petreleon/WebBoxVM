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

- [ ] Map each required ID to its audited root, members, selection configuration, and core-scope limit.
- [ ] Design a bounded logical-closure identity that binds every required member and generated input.
- [ ] Define explicit pre/post admission rules so audit candidates remain rejected after a valid cutover.
- [ ] Add focused hostile tests for missing, stale, oversize, duplicate, or scope-expanded closure members.
- [ ] Record GLES-extension and Vulkan WSI/video/extension exclusions; keep cutover open if scope is unresolved.

## Verification

- A root alone never satisfies a compound source requirement, even if its URL, revision, and digest match.
- The model preserves F02.2's byte, cache, family, and immutable-source rules without claiming support.
