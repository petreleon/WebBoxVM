# F02.2.2 — Prove verifier behavior with isolated fixtures

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.2.2
Depends: F02.2.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.2.1](../01-fetch-contract/README.md).

## Outcome

Hermetic fixtures demonstrate that the verifier accepts one correct payload and fails closed on each
required bad input without contacting a public upstream.

## Starting points

- [fetch contract](../01-fetch-contract/README.md)
- [input inventory](../../01-input-inventory/README.md)
- [graphics workflow](../../../../../workflow.md)

## Checklist

- [x] Add a local success fixture with independently known byte count and SHA-256.
- [x] Cover wrong SHA-256, revision metadata, unavailable input, malformed input, and redirect denial.
- [x] Assert failed cases leave no accepted cache entry and a successful case is re-hashable offline.
- [x] Keep fixtures and tests modular, deterministic, and at most 180 physical lines per file.

## Verification

- The hermetic suite reports a nonzero positive and negative case count.
- Every rejected fixture names the failing condition and leaves the cache unaccepted.

## Fixture boundary

`fixture_transport.py` supplies only in-memory response streams and injected opener outcomes; it does
not start a server or open a socket. `fixture_transport_test.py` passes those fakes into F02.2.1's
`fetch_to_cache(..., opener=...)` seam, retaining the production HTTPS host, revision, redirect, and
hash policies unchanged.

Run the suite with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/02-hermetic-fixtures/fixture_transport_test.py
```

This leaf does not invoke `source_fetch.py`; F02.2.3 owns its historical 15-input fetch and
F02.3.3.4.1.2 owns the grammar renewal's 16-input re-fetch.
