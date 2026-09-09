# F02.4.4.2 — Cut over inventory and candidate locks

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.4.4.2
Depends: F02.3.1, F02.3.4, F06, F02.4.4.1
Evidence: pending

Prerequisite lists: [F02.1](../../../01-input-inventory/README.md),
[F02.3.4](../../../03-provenance-contract/04-provenance-validation/README.md), and the admission shape.

## Outcome

After F02.4.4.1 admits every required logical closure, one bounded inventory transaction grows the
17-entry inventory to its exact reviewed successor, regenerates `inventory.lock`, and renews every
candidate audit's lock binding. No root is added as a substitute for its unresolved closure.

## Starting points

- [inventory layout](../../../01-input-inventory/inventory_layout.py)
- [inventory components](../../../01-input-inventory/inputs)
- [candidate audits](../../01-opengl-input-audit/candidates.json)
- [fetch policy](../../../02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [ ] Add only F02.4.4.1-admitted logical identities in bounded ordered inventory components.
- [ ] Regenerate the canonical lock and prove omission, reorder, family, and raw-byte changes fail closed.
- [ ] Renew all three audit `inventory_sha256` bindings in the same transaction.
- [ ] Extend loaders and fetch fixtures in new bounded modules where current files are at the line cap.
- [ ] Reject stale pre-cutover candidates and partial logical closures before cache or provenance actions.

## Verification

- The successor lock has one deterministic identity and no consumer can combine it with an old audit.
- This changes source provenance only; it does not run suites or change a profile to supported.
