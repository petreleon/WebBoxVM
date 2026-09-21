# F03.3.2.2.5 — Aggregate command/state coverage and handoff

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.3.2.2.5
Depends: F03.3.2.2.3, F03.3.2.2.4
Evidence: pending

## Outcome

One self-hashed receipt joins closed command/state slices, rejects missing or overlapping family coverage, and
hands off raw facts without turning them into Matrix rows, ownership, or test obligations.

## Starting points

- [object/resource command slices](../03-object-resource-command-slices/README.md)
- [state/execution command slices](../04-state-execution-command-slices/README.md)
- [parent raw inventory](../README.md)

## Checklist

- [ ] Require exact child receipts, cache/source identities, grammar, and domain-classification map.
- [ ] Bind every classified command/state family exactly once or preserve its explicit non-command route.
- [ ] Reject missing, duplicate, overlapping, reordered, stale, cross-profile, or promoted child observations.
- [ ] Preserve raw-only facts with zero Matrix rows, CTS runs, ownership, reference-test, or claim fields.
- [ ] Attach the aggregate receipt; only it may close F03.3.2.2.

## Verification

The aggregate proves only source-inventory coverage. It does not prove GLES behavior, guest/browser execution,
support, conformance, certification, or performance.
