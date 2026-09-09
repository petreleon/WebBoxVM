# F02.2 — Verify isolated source fetches and hashes

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F02.2
Depends: F02.1
Evidence: pending

Prerequisite lists: [F02.1](../01-input-inventory/README.md).

## Outcome

A clean cache can fetch each pinned input, verify its bytes, and fail closed on corruption.

## Starting points

- [input inventory](../01-input-inventory/README.md)
- [baseline commands](../../../../baseline.md)
- [source-file limit test](../../../../../../emulator/tests/source_file_limits.rs)

## Checklist

- [ ] [F02.2.1 — Define the fail-closed fetch and cache contract](01-fetch-contract/README.md)
- [ ] [F02.2.2 — Prove verifier behavior with isolated fixtures](02-hermetic-fixtures/README.md)
- [ ] [F02.2.3 — Verify the live immutable inventory](03-live-inventory/README.md)

## Verification

- A clean temporary build fetches every pinned entry and a deliberately wrong hash fails.
- Unit tests prove that a network failure, redirect policy violation, or malformed manifest cannot
  produce an accepted cache entry.
