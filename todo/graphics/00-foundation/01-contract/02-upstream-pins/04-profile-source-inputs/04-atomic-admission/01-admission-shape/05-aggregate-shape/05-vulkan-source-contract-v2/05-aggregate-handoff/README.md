# F02.4.4.1.5.5.5 — Hand off the V2 aggregate

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.5.5
Depends: F02.4.4.1.5.5.3, F02.4.4.1.5.5.4
Evidence: pending

Prerequisite lists: [the external cache proof](../03-external-closure-cache/README.md),
[the taxonomy](../04-coverage-taxonomy/README.md), and the [aggregate proof](../../04-closure-admission-proof/README.md).

## Outcome

One V2 adapter presents a verified canonical-suite receipt to the aggregate admission proof while
leaving the V1 inventory, contracts, and blocker evidence read-only until a later atomic cutover.

## Starting points

- [aggregate proof](../../04-closure-admission-proof/README.md)
- [F02 inventory cutover](../../../../02-inventory-cutover/README.md)
- [F03 gate transition](../../../../05-f03-gate/README.md)

## Checklist

- [ ] Bind the V2 root, closure ledger, cache receipt, and taxonomy digest into one handoff identity.
- [ ] Require all V2 receipt fields before the aggregate proof can consume the suite.
- [ ] Reject V1/V2 mixing, stale root identity, partial closure, and taxonomy/ledger mismatch.
- [ ] Document that Docs generated artifacts remain outside the admitted implementation-source closure.
- [ ] Run focused adapter tests and attach a receipt for F02.4.4.1.5.4.

## Verification

- The handoff changes neither the inventory nor F03 state; those remain separate atomic tasks.
- A successful handoff has no compatibility, conformance, certification, or performance implication.
