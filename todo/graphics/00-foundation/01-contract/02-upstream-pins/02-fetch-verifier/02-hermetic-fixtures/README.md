# F02.2.2 — Prove verifier behavior with isolated fixtures

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.2.2
Depends: F02.2.1
Evidence: pending

Prerequisite lists: [F02.2.1](../01-fetch-contract/README.md).

## Outcome

Hermetic fixtures demonstrate that the verifier accepts one correct payload and fails closed on each
required bad input without contacting a public upstream.

## Starting points

- [fetch contract](../01-fetch-contract/README.md)
- [input inventory](../../01-input-inventory/README.md)
- [graphics workflow](../../../../../workflow.md)

## Checklist

- [ ] Add a local success fixture with independently known byte count and SHA-256.
- [ ] Cover wrong SHA-256, revision metadata, unavailable input, malformed input, and redirect denial.
- [ ] Assert failed cases leave no accepted cache entry and a successful case is re-hashable offline.
- [ ] Keep fixtures and tests modular, deterministic, and at most 180 physical lines per file.

## Verification

- The hermetic suite reports a nonzero positive and negative case count.
- Every rejected fixture names the failing condition and leaves the cache unaccepted.
