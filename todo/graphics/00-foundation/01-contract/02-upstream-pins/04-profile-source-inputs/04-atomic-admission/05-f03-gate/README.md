# F02.4.4.5 — Transition the F03 source gate

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.4.4.5
Depends: F03.1, F02.4.4.1, F02.4.4.2, F02.4.4.4
Evidence: pending

Prerequisite lists: [F03.1](../../../../03-feature-matrix/01-profile-scope/README.md), the
closure-aware admission model, and the fresh-cache proof.

## Outcome

Only after a closure-aware inventory and fresh cache exist, F03's lock-bound inputs may transition
from `inventory-sources-incomplete` to `matrix-incomplete`. The gate proves source availability, not
feature semantics, runtime execution, conformance, or performance.

## Starting points

- [F03 profile contract](../../../../03-feature-matrix/01-profile-scope/profile_contract.py)
- [F03 scope JSON](../../../../03-feature-matrix/01-profile-scope/profile_scope.json)
- [source requirements](../../../../03-feature-matrix/01-profile-scope/source_requirements.json)
- [F03 validation tests](../../../../03-feature-matrix/01-profile-scope/validate_profile_scope_test.py)

## Checklist

- [ ] Make F03 require an admitted closure, not only a singular source ID, for each required role.
- [ ] Renew lock-bound source JSON and preserve the canonical profile/requirement ordering.
- [ ] Transition only the source blocker to `matrix-incomplete` after the fresh-cache proof.
- [ ] Add positive and hostile tests for stale locks, root-only substitutes, and mixed closures.
- [ ] Keep every API feature row non-supported until its separate matrix and implementation evidence exists.

## Verification

- F03 rejects a root-only, stale, partial, or scope-expanded source even when its ID is present.
- A passing F03 source gate does not claim OpenGL, GLES, Vulkan, guest, browser, conformance, or speed.
