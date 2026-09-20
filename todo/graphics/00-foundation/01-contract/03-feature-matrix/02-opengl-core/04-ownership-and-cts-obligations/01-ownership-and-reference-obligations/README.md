# F03.2.4.1 — Assign ownership and reference obligations

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.2.4.1
Depends: F03.2.2, F03.2.3
Evidence: pending

## Outcome

Every admitted raw OpenGL fact receives either a concrete downstream implementation owner and independent
reference obligation or a visible, specific blocked reason. Raw source extraction alone cannot supply
either field.

## Starting points

- [command/object raw handoff](../../02-command-object-state-inventory/README.md)
- [limit/format/shader inventory](../../03-limit-format-shader-inventory/README.md)
- [admitted full-suite root](../../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)

## Checklist

- [ ] Join complete raw inventory families and reject a missing, duplicate, or source-incompatible fact.
- [ ] Assign each mandatory fact a concrete implementation task or an explicit unresolved-owner blocker;
  never use a generic transport milestone as an owner.
- [ ] Assign an independent native, guest, or canonical conformance obligation with an exact selector or
  documented blocked reason.
- [ ] Classify bounded VirGL evidence only when it supports the exact fact; do not replace it with a lower
  API version or inferred support.
- [ ] Reject an empty owner, generic obligation, stale suite root, or mismatched profile before matrix
  import.
- [ ] Attach a no-claim ownership and reference-obligation receipt.

## Verification

This child maps planned ownership and reference obligations only. It does not execute CTS, prove a guest
or browser path, certify an implementation, or change any row from blocked.
