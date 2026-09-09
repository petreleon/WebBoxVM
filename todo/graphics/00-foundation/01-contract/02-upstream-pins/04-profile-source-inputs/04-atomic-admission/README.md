# F02.4.4 — Atomically admit sources and renew lock consumers

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.4.4
Depends: F02.1, F02.2, F02.3.1, F02.3.4, F02.4.1, F02.4.2, F02.4.3, F06
Evidence: pending

Prerequisite lists: [F02.3.4](../../03-provenance-contract/04-provenance-validation/README.md),
the three source audits, and [F06](../../../../02-reproducibility/03-file-layout/README.md).

## Outcome

A future atomic cutover may admit all six reviewed logical sources only after every compound source
has a closure-aware model that satisfies F02.2. Rejected audit roots cannot enter the inventory merely
because their top-level URL is pinned. Once that condition is met, the inventory, provenance,
generator, reproducibility, cache, and F03.1 consumers renew in one validated cutover.

## Starting points

- [inventory loader](../../01-input-inventory/inventory_layout.py)
- [provenance closure](../../03-provenance-contract/04-provenance-validation/validate_provenance_closure.py)
- [F03.1 source contract](../../../03-feature-matrix/01-profile-scope/profile_contract.py)

## Checklist

- [ ] [F02.4.4.1 — Define closure-aware admission shape](01-admission-shape/README.md)
- [ ] [F02.4.4.2 — Cut over inventory and candidate locks](02-inventory-cutover/README.md)
- [ ] [F02.4.4.3 — Renew provenance and reproducibility consumers](03-provenance-consumers/README.md)
- [ ] [F02.4.4.4 — Prove a fresh complete external cache](04-fresh-cache-proof/README.md)
- [ ] [F02.4.4.5 — Transition the F03 source gate](05-f03-gate/README.md)

## Verification

- No consumer accepts an old lock, stale source identity, partial cache, or pre-cutover generated data.
- The cutover proves source provenance only: it does not mark a graphics profile, guest API, browser
  route, conformance run, or performance result as supported.

## Split rationale

The audits showed that three roots are compound and rejected, while F03 currently recognizes only a
singular source ID. Schema design must therefore precede any lock rewrite. The five children isolate
the new admission boundary, atomic data cutover, downstream consumers, fresh-cache proof, and F03
transition so a partial update cannot turn a rejected root into a claimed profile input.
