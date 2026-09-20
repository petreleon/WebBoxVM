# F03.2.2.4 — Handoff raw facts and guard matrix import

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.2.2.4
Depends: F03.2.2.2, F03.2.2.3
Evidence: pending

## Outcome

The command/object and state/lifecycle raw inventories form one self-consistent handoff for F03.2.4. A
fail-closed import guard proves that raw source facts cannot masquerade as Matrix v2 rows before ownership
and independent reference obligations exist.

## Starting points

- [command and object raw inventory](../02-command-object-raw-inventory/README.md)
- [state and lifecycle raw inventory](../03-state-and-lifecycle-raw-inventory/README.md)
- [ownership and CTS parent](../../04-ownership-and-cts-obligations/README.md)

## Checklist

- [ ] Aggregate exact source identity, ordered raw counts, hashes, source scopes, and exclusions.
- [ ] Reject a missing family, cross-profile source, stale cache identity, duplicate fact, or reordered
  handoff.
- [ ] Refuse Matrix v2 emission while an implementation owner or independent reference obligation is
  absent.
- [ ] Keep every raw fact blocked and retain `matrix-incomplete` with zero CTS executions and no claims.
- [ ] Provide a precise F03.2.4 handoff and attach a no-claim receipt.

## Verification

The handoff is deliberately not a `matrix_contract_v2.py` document. F03.2.4.2 may import it only after
F03.2.4.1 records the missing ownership and test-plan facts.
