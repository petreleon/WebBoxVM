# F02.4.4.1.5.2.3.5.5 — Restage and cross-prove

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.5.5
Depends: F02.4.4.1.5.2.3.5.4, F02.4.4.1.5.2.3.4
Evidence: pending

Prerequisite lists: the [actual manifest](../04-reify-actual-member-closure/README.md),
[staged cache](../../04-stage-build-witnesses/README.md), and [Docs binding parent](../../README.md).

## Outcome

Restage and rehash the actual-grammar closure in a fresh private cache, then cross-prove it against scope, witness,
two fresh lineage captures, both output trees, and one strict unadmitted marker.

## Starting points

- [cache marker contract](../../04-stage-build-witnesses/04-verify-staged-cache/vulkan_docs_cache_marker_contract.py)
- [actual closure parser](../../02-actual-closure-identity/vulkan_docs_identity_contract.py)
- [fresh lineage receipt](../03-capture-fresh-lineage/README.md)

## Checklist

- [ ] Rebuild the plan from the actual manifest and re-read all providers for 1,760 inputs and both output trees.
- [ ] Publish and reopen the strict marker only after exact closure verification and post-publication rehash.
- [ ] Cross-bind manifest, scope, witness, lineage identities, receipts, and marker while rejecting stale/raced substitutes.
- [ ] Run focused positive/hostile tests, source limits, `make test`, diff check, and roadmap checker.
- [ ] Record the unadmitted closure proof and aggregate parent PASS only if every required check succeeds.

## Verification

- This may complete only the Docs source-closure parent; the independent VCTS closure remains mandatory.
- No inventory, guest API, browser, CTS, conformance, or performance claim changes through this task.
