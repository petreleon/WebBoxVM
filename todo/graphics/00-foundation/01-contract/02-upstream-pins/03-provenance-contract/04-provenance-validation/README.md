# F02.3.4 — Verify provenance closure

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.3.4
Depends: F02.2, F02.3.1, F02.3.2, F02.3.3
Evidence: pending

Prerequisite lists: [F02.2](../../02-fetch-verifier/README.md),
[F02.3.2](../02-abi-fixtures/README.md), and [F02.3.3](../03-generator-outputs/README.md).

## Outcome

The completed provenance records are checked against the fresh verified cache before F02 can close.

## Starting points

- [live inventory verification](../../02-fetch-verifier/03-live-inventory/README.md)
- [record contract](../01-provenance-record/README.md)
- [ABI records](../02-abi-fixtures/README.md)
- [generator records](../03-generator-outputs/README.md)

## Checklist

- [ ] Re-run provenance validation after F02.2's clean external-cache fetch and offline rehash.
- [ ] Verify ABI and generator samples against the recorded manifest revision, IDs, digests, licenses,
  commands, and output hashes.
- [ ] Record the fresh-fetch/provenance output, nonzero test count, and unavailable input as failure.
- [ ] Run focused tests, source limits, roadmap, whitespace, and required full gates before the receipt.

## Verification

- Every sample resolves only to an entry accepted by F02.2's fresh cache result.
- A missing or unavailable input keeps F02.3 and its parent F02 open rather than producing a pass.
