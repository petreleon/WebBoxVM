# F02.4.4.1.5 — Aggregate the admission shape

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.4.4.1.5
Depends: F02.4.4.1.1, F02.4.4.1.2, F02.4.4.1.3, F02.4.4.1.4
Evidence: pending

Prerequisite lists: every preceding admission-shape child and
[F02.2](../../../../02-fetch-verifier/README.md).

## Outcome

One versioned, bounded model reconciles every required-source shape and only exposes a cutover-ready
result if each closure is complete, scoped, and valid under its explicit policy. V1 blocker records
remain historical evidence; V2 is the only active route to a later source cutover.

## Starting points

- [source-map child](../02-source-map/README.md)
- [Vulkan-boundaries child](../03-vulkan-boundaries/README.md)
- [post-cutover rules child](../04-post-cutover-rules/README.md)

## Checklist

- [x] [F02.4.4.1.5.1 — Aggregate the current pre-admission state](01-pre-admission-aggregate/README.md)
- [x] [F02.4.4.1.5.2 — Historical Vulkan Docs closure policy](02-vulkan-docs-closure/README.md)
- [x] [F02.4.4.1.5.3 — Historical VCTS core-selector policy](03-vcts-core-closure/README.md)
- [x] [F02.4.4.1.5.5 — Define the Vulkan source-contract V2](05-vulkan-source-contract-v2/README.md)
- [ ] [F02.4.4.1.5.4 — Prove the aggregate closure admission](04-closure-admission-proof/README.md)

## Verification

- This parent may complete only when every required source is genuinely modelled; V2 may not turn a
  superseded V1 record into PASS or conceal an unresolved source, suite, or classification boundary.
