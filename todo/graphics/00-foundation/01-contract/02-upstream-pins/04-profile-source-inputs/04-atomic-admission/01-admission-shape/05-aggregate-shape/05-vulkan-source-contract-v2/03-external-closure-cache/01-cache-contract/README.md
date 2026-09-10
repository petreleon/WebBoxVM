# F02.4.4.1.5.5.3.1 — Define the streaming cache contract

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.5.3.1
Depends: F02.2.3, F02.4.4.1.5.5.2
Evidence: [receipt](evidence.md)

Prerequisite lists: [the V2 schema](../../02-canonical-suite-schema/README.md) and
[the F02 live-fetch proof](../../../../../../../02-fetch-verifier/03-live-inventory/README.md).

## Outcome

A hermetic contract streams one pinned selector and its declared members to an external,
content-addressed cache, atomically publishes only a self-hashed receipt, and rehashes it offline.
It does not claim a real VCTS download.

## Starting points

- [V2 root identity](../../02-canonical-suite-schema/vcts_root_identity.json)
- [V2 ledger schema](../../02-canonical-suite-schema/canonical_suite_ledger.py)
- [F02 transport policy](../../../../../../../02-fetch-verifier/01-fetch-contract/source_cache.py)

## Checklist

- [x] Stream HTTPS identity bytes without redirects or in-memory aggregate payloads.
- [x] Validate an external, non-symlink, content-addressed target before atomic publication.
- [x] Bind the root, ordered ledger, member Git/SHA-256 identities, limits, and cache receipt.
- [x] Rehash offline and reject path escape, redirects, stale/partial cache, and limit violations.
- [x] Add hermetic positive and hostile tests; retain no synthetic full suite in Git.

## Verification

- A green fixture proves cache mechanics only. It cannot substitute for the live child or CTS execution.
