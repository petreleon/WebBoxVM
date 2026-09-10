# F02.4.4.4 — Prove a fresh complete external cache

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.4.4.4
Depends: F02.2.3, F02.3.4, F02.4.4.2, F02.4.4.3
Evidence: pending

Prerequisite lists: [live inventory verification](../../../02-fetch-verifier/03-live-inventory/README.md),
[provenance validation](../../../03-provenance-contract/04-provenance-validation/README.md), and the renewed consumers.

## Outcome

A newly created external cache proves every admitted logical source and its closure can be fetched,
bounded, rehashed offline, and consumed by the revised provenance validator without reusing prior data.

## Starting points

- [live fetch runner](../../../02-fetch-verifier/01-fetch-contract/source_fetch.py)
- [external-cache policy](../../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [revised provenance validator](../../../03-provenance-contract/04-provenance-validation/validate_provenance_closure.py)

## Checklist

- [ ] Fetch every new identity and required closure member into a newly created external cache.
- [ ] Verify the root's declared bytes, SHA-256, license, selector boundary, and size policy; verify
  each V2 suite member by its declared immutable identity rather than inventing a member license.
- [ ] Rehash offline and reject incomplete, stale, redirected, oversize, or scope-expanded closures.
- [ ] Run the revised provenance closure against that cache and record exact nonzero results.
- [ ] Keep all upstream payloads and large generated material outside the repository.

## Verification

- A prior cache cannot make a missing member, stale digest, or partial logical source pass.
- Passing this proof establishes source provenance only, never API compatibility or performance.
