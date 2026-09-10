# F02.4.4.1.5.5.4 — Freeze coverage-report taxonomy

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.5.4
Depends: F02.4.4.1.5.5.2, F03.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [the V2 schema](../02-canonical-suite-schema/README.md) and
[F03 profile scope](../../../../../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

A hash-bound reporting taxonomy distinguishes Vulkan 1.4 core evidence from WSI, video, extension,
and unknown cases while keeping the Khronos suite root and ordering unchanged.

## Starting points

- [VCTS V1 boundary](../../../03-vulkan-boundaries/boundaries.json)
- [F03 target scope](../../../../../../../03-feature-matrix/01-profile-scope/profile_scope.json)
- [V2 schema](../02-canonical-suite-schema/README.md)
- [frozen taxonomy](coverage_taxonomy.json) · [validator](coverage_taxonomy.py)
- [diagnostic report contract](coverage_report.py) · [hermetic tests](coverage_taxonomy_test.py)

## Maintained contract

`coverage_taxonomy.json` has only six explicit non-core rules: the V1-observed `wsi`, `video`,
`cooperative-vector`, `data-graph`, `ray-query`, and `tensor` selectors. It deliberately has no
name-based `core` rule: all other direct and recursive ledger members are `unknown`, hence non-core,
until a separately reviewed test-to-spec mapping exists. Rules and each emitted member locator bind
the peeled VCTS commit; `coverage_report.py` binds the ordered, unfiltered ledger digest and each
member hash to the taxonomy digest.

Both report modes retain every selected member. `core-readiness` remains blocked with no reviewed core
map; `complete-suite-diagnostics` can record test, skip, and failure counts but never reports Khronos
conformance or certification. A revised core map must be a reviewed versioned taxonomy change, never a
local run override.

## Checklist

- [x] Define exhaustive `core`, `wsi`, `video`, `extension`, and `unknown` reporting categories.
- [x] Bind every classification to the pinned suite ledger and a reviewable rule or source locator.
- [x] Treat unknown as non-core until reviewed; preserve selector order and all selected members.
- [x] Define reports for core readiness and complete-suite diagnostics without a certification claim.
- [x] Add hostile tests for missing, duplicate, stale, reclassified, and silently filtered members.

## Verification

- No local category file changes the selector that runs or turns a partial run into Khronos conformance.
- Every core readiness result identifies its suite revision, taxonomy digest, tests, skips, and failures.
