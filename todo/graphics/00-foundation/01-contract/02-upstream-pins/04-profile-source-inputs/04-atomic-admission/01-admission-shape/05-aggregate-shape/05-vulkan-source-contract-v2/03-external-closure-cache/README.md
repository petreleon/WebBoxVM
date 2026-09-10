# F02.4.4.1.5.5.3 — Verify the external closure cache

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.5.3
Depends: F02.2.3, F02.4.4.1.5.5.2
Evidence: pending

Prerequisite lists: [the V2 schema](../02-canonical-suite-schema/README.md) and
[F02 live fetch proof](../../../../../../02-fetch-verifier/03-live-inventory/README.md).

## Outcome

An external, content-addressed, streaming cache proves the V2 closure without importing hundreds of
megabytes into the repository or holding it in memory. A receipt binds the cache to the whole ledger.

## Starting points

- [external-cache policy](../../../../../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [source fetch runner](../../../../../../02-fetch-verifier/01-fetch-contract/source_fetch.py)
- [VCTS boundary record](../../03-vcts-core-closure/evidence.md)

## Checklist

- [ ] Download and hash each declared member by stream into a temporary external-cache target.
- [ ] Require atomic rename only after byte count and hash match the same pinned commit ledger.
- [ ] Rehash offline and bind ordered members, include edges, limits, and aggregate digest in a receipt.
- [ ] Reject redirects, symlinks, path escape, partial cache, stale cache, and resource-limit violations.
- [ ] Keep the full suite outside Git and attach focused positive and hostile evidence.

## Verification

- A prior cache cannot make a missing, altered, redirected, or out-of-limit member pass.
- Passing establishes reproducible test-suite input availability, never guest compatibility or speed.
