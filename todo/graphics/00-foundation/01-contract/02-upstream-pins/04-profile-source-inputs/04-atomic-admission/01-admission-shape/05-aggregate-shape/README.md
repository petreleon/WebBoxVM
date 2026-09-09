# F02.4.4.1.5 — Aggregate the admission shape

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.4.4.1.5
Depends: F02.4.4.1.1, F02.4.4.1.2, F02.4.4.1.3, F02.4.4.1.4
Evidence: pending

Prerequisite lists: every preceding admission-shape child and
[F02.2](../../../../02-fetch-verifier/README.md).

## Outcome

One bounded model reconciles every required-source shape and only exposes a cutover-ready result if
all compound closures are complete, scoped, and policy-valid. Until then, it publishes exact blockers.

## Starting points

- [source-map child](../02-source-map/README.md)
- [Vulkan-boundaries child](../03-vulkan-boundaries/README.md)
- [post-cutover rules child](../04-post-cutover-rules/README.md)

## Checklist

- [ ] Verify all six required IDs have exactly one valid source-shape record.
- [ ] Verify every compound member/generated input and exclusion has an immutable bounded identity.
- [ ] Verify no unresolved root, stale lock, or partial state has a cutover-ready result.
- [ ] Attach the aggregate receipt with focused positive and hostile results.

## Verification

- This parent may complete only when every required source is genuinely modelled; a Vulkan blocker
  remains visible and prevents the F02.4.4.2 inventory cutover.
