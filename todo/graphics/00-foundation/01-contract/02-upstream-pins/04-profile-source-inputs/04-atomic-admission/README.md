# F02.4.4 — Atomically admit sources and renew lock consumers

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.4.4
Depends: F02.1, F02.2, F02.3.1, F02.3.4, F02.4.1, F02.4.2, F02.4.3, F06
Evidence: pending

Prerequisite lists: [F02.3.4](../../03-provenance-contract/04-provenance-validation/README.md),
the three source audits, and [F06](../../../../02-reproducibility/03-file-layout/README.md).

## Outcome

All six reviewed sources enter one regenerated inventory lock and every provenance, generator, and
F03.1 consumer is renewed in the same validated cutover.

## Starting points

- [inventory loader](../../01-input-inventory/inventory_layout.py)
- [provenance closure](../../03-provenance-contract/04-provenance-validation/validate_provenance_closure.py)
- [F03.1 source contract](../../../03-feature-matrix/01-profile-scope/profile_contract.py)

## Checklist

- [ ] Add the six admitted identities in a new inventory component and renew the canonical lock.
- [ ] Extend the family/schema and fetch validation only with focused hostile-fixture coverage.
- [ ] Renew every affected provenance sidecar, generator fixture, deterministic chunk, and count assertion.
- [ ] Fetch and offline-rehash a fresh complete cache, then verify the revised provenance closure.
- [ ] Renew F03.1 lock-bound JSON, transition its blocker to `matrix-incomplete`, and verify its gate.
- [ ] Run focused/full/limit/roadmap/whitespace checks and attach the atomic admission receipt.

## Verification

- No consumer accepts an old lock, stale source identity, partial cache, or pre-cutover generated data.
- The cutover proves source provenance only: it does not mark a graphics profile, guest API, browser
  route, conformance run, or performance result as supported.
