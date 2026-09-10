# F02.4.4.1.5.4.4.4 — Reconcile the final aggregate admission

[Parent task](../README.md)

Task: F02.4.4.1.5.4.4.4
Depends: F02.4.4.1.5.4.4.2, F02.4.4.1.5.4.4.3
Evidence: pending

Prerequisite lists: the [derived Docs closure](../02-derived-docs-closure/README.md), the
[source/release boundary](../03-source-release-boundary/README.md), and the
[blocked-state receipt](../../03-blocked-state-receipt/README.md).

## Outcome

One versioned admission reconciler may accept the successor Docs envelope only with all six native
role closures, their immutable identities, and truthful source/release boundaries. It may make an
admission decision only after every independent GLES and VCTS blocker has changed through its own policy.

## Starting points

- [frozen current-state contract](../../01-proof-contract/README.md)
- [current evidence reconciler](../../02-reconcile-current-evidence/README.md)
- [F02 atomic transition](../../../../04-post-cutover-rules/README.md)

## Checklist

- [ ] Version the admission grammar without mutating or reinterpreting the frozen blocked-state evidence.
- [ ] Require exact six-role closure identities and reject mixed V1/V2/successor envelopes, aliases, and stale inputs.
- [ ] Preserve complete canonical-suite scope reporting and reject VCTS-as-Docs or local core-selector substitution.
- [ ] Return `admission_eligible=true` only when every required closure is genuinely valid; otherwise retain the first blocker.
- [ ] Keep inventory, fresh-cache, cutover, F03, support, certification, and performance implications separate.
- [ ] Run focused positive/hostile tests and attach exact PASS evidence before any later atomic transition.

## Verification

- A passing grammar cannot create evidence; current `blocked` receipts remain valid historical state.
- The final aggregate remains blocked until the independent GLES and VCTS closure conditions are satisfied.
