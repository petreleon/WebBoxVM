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

- [ ] [F02.4.4.1.5.1 — Aggregate the current pre-admission state](01-pre-admission-aggregate/README.md)
- [ ] [F02.4.4.1.5.2 — Resolve the Vulkan Docs core closure](02-vulkan-docs-closure/README.md)
- [ ] [F02.4.4.1.5.3 — Resolve the VCTS Vulkan-core closure](03-vcts-core-closure/README.md)
- [ ] [F02.4.4.1.5.4 — Prove the aggregate closure admission](04-closure-admission-proof/README.md)

## Verification

- This parent may complete only when every required source is genuinely modelled; a Vulkan blocker
  remains visible and prevents the F02.4.4.2 inventory cutover.
