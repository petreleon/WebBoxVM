# F02.4.4.1.5.5.4 — Freeze coverage-report taxonomy

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.5.4
Depends: F02.4.4.1.5.5.2, F03.1
Evidence: pending

Prerequisite lists: [the V2 schema](../02-canonical-suite-schema/README.md) and
[F03 profile scope](../../../../../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

A hash-bound reporting taxonomy distinguishes Vulkan 1.4 core evidence from WSI, video, extension,
and unknown cases while keeping the Khronos suite root and ordering unchanged.

## Starting points

- [VCTS V1 boundary](../../../03-vulkan-boundaries/boundaries.json)
- [F03 target scope](../../../../../../../03-feature-matrix/01-profile-scope/profile_scope.json)
- [V2 schema](../02-canonical-suite-schema/README.md)

## Checklist

- [ ] Define exhaustive `core`, `wsi`, `video`, `extension`, and `unknown` reporting categories.
- [ ] Bind every classification to the pinned suite ledger and a reviewable rule or source locator.
- [ ] Treat unknown as non-core until reviewed; preserve selector order and all selected members.
- [ ] Define reports for core readiness and complete-suite diagnostics without a certification claim.
- [ ] Add hostile tests for missing, duplicate, stale, reclassified, and silently filtered members.

## Verification

- No local category file changes the selector that runs or turns a partial run into Khronos conformance.
- Every core readiness result identifies its suite revision, taxonomy digest, tests, skips, and failures.
