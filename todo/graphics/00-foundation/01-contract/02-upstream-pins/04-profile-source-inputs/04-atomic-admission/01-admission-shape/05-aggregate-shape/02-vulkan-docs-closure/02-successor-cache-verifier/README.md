# F02.4.4.1.5.2.2 — Verify successor closure cache staging

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.2
Depends: F02.4.4.1.5.2.1
Evidence: pending

Prerequisite lists: the [successor identity child](../01-successor-identity/README.md),
[F02.2 cache rules](../../../../../../02-fetch-verifier/01-fetch-contract/source_cache.py), and the
[Docs blocker record](../evidence.md).

## Outcome

An isolated successor cache verifier stages raw and generated closure members outside the repository and publishes
a reusable closure marker only after every ordered member and derived output rehashes successfully.

## Starting points

- [current cache verifier](../../../../../../02-fetch-verifier/01-fetch-contract/source_cache.py)
- [successor identity child](../01-successor-identity/README.md)
- [F02.2 fetch tests](../../../../../../02-fetch-verifier/01-fetch-contract/source_fetch_test.py)

## Checklist

- [ ] Stage raw and generated fixture members under an explicit external cache root.
- [ ] Require every member's digest, bytes, recipe/configuration, and toolchain identity before completion.
- [ ] Rehash reused members and reject partial, stale, reordered, or cross-closure completion markers.
- [ ] Keep active F02.2 cache paths and loaders untouched while proving the isolated successor path.
- [ ] Add hermetic positive/hostile tests and a focused PASS receipt.

## Verification

- A partial member cache is never a closure cache hit, and no active inventory input can be reinterpreted.
- This child supplies only isolated cache evidence; it cannot admit Docs or change guest/browser behavior.
