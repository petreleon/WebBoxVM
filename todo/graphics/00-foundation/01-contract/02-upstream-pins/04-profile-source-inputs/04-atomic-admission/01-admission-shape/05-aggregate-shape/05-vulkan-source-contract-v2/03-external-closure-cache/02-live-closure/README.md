# F02.4.4.1.5.5.3.2 — Capture the live closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.5.3.2
Depends: F02.4.4.1.5.5.3.1, F02.4.4.1.5.5.2
Evidence: [receipt](evidence.md)

Prerequisite lists: [the cache contract](../01-cache-contract/README.md) and
[the V2 selector identity](../../02-canonical-suite-schema/vcts_root_identity.json).

## Outcome

A real immutable-Khronos fetch streams every ordered selector member to a disposable external cache,
records the validated ledger/receipt metadata, then proves the same cache offline.

## Starting points

- [cache-contract task](../01-cache-contract/README.md)
- [VCTS boundary record](../../../03-vcts-core-closure/evidence.md)
- [live-fetch evidence format](../../../../../../../02-fetch-verifier/03-live-inventory/evidence.md)

## Checklist

- [x] Fetch the 3,347-byte root and every one of its 98 pinned direct members by streamed HTTPS.
- [x] Record exact bytes, SHA-256, Git blob SHA-1, ordered ledger digest, aggregate bytes, and cache receipt.
- [x] Re-run offline verification against the same external cache with no transport requests.
- [x] Record the exact first network/cache failure as BLOCKED or FAIL; do not fabricate an unavailable member.
- [x] Keep payloads outside Git and attach reproducible receipt evidence with no API/performance claim.

## Verification

- Passing proves only reproducible CTS suite input availability at the V2 pin; it never proves core
  compatibility, a complete CTS run, Khronos conformance, browser execution, or near-native speed.
