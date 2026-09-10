# F02.4.4.1.5.2.3.5.1 — Authoritative member metadata

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.5.1
Depends: F02.4.4.1.5.2.3.3.2, F02.4.4.1.5.2.3.3.3
Evidence: [blocker receipt](evidence.md)

Prerequisite lists: the [actual Docs grammar](../../02-actual-closure-identity/README.md),
[captured records](../../03-capture-core-closure/README.md), and the [blocker record](../evidence.md).

## Outcome

Define and validate the single authoritative mapping for every static actual-grammar member field without claiming
that a generated payload is licensed, cached, or produced unless the pinned evidence proves that fact.

## Starting points

- [identity member parser](../../02-actual-closure-identity/vulkan_docs_identity_members.py)
- [pinned source observer](../../03-capture-core-closure/01-observe-pinned-build-inputs/README.md)
- [scope semantics](../../03-capture-core-closure/02-bind-core-input-scope/vulkan_docs_scope_semantics.py)

## Checklist

- [ ] Inventory authoritative evidence for every ID/license/cache/role/provenance field plus raw source/URL/revision and derived generation.
- [ ] Canonicalize collision-free IDs and required cache spellings without implying current cache residency.
- [ ] Bind raw fields to the exact pinned source tree and require an explicit source for every derived field.
- [ ] Reject missing, ambiguous, defaulted, stale, or source/output-swapped metadata with focused tests.
- [ ] Record an unadmitted receipt, or the first metadata field that has no authoritative source.

## Verification

- SPDX/REUSE evidence must be evaluated against the full pinned source tree; it is not a runtime input observation.
- A uniform generated license, role, or provenance label is forbidden unless its source and applicability are explicit.
- Status: **BLOCKED**. Raw static facts are reified from the sealed source tree, but no authority establishes the
  first missing derived static field, `license`, for all 1,462 generated members. This child remains unchecked.
