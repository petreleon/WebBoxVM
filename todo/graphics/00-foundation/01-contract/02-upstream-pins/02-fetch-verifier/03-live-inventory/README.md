# F02.2.3 — Verify the live immutable inventory

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.2.3
Depends: F02.2.1, F02.2.2
Evidence: pending

Prerequisite lists: [F02.2.1](../01-fetch-contract/README.md) and
[F02.2.2](../02-hermetic-fixtures/README.md).

## Outcome

A fresh external temporary cache proves every reviewed manifest input still has its pinned bytes,
hash, license record, and declared immutable identity.

## Starting points

- [input inventory](../../01-input-inventory/README.md)
- [fetch contract](../01-fetch-contract/README.md)
- [fixture evidence](../02-hermetic-fixtures/README.md)
- [graphics workflow](../../../../../workflow.md)

## Checklist

- [ ] Fetch every manifest input into a newly created external temporary cache.
- [ ] Record exact command output, cache paths, SHA-256 values, byte counts, and licenses.
- [ ] Re-hash the populated cache offline and record any unavailable source as a failure/blocker.
- [ ] Run final verifier, source-limit, roadmap, and whitespace checks before its receipt.

## Verification

- The live run has one successful verified result per manifest entry and no accepted mismatch.
- A missing or changed upstream source leaves F02.2 open rather than becoming a passing result.
