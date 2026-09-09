# F02.3 — Bind ABI fixtures and generators to the manifest

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F02.3
Depends: F02.1, F02.2
Evidence: pending

Prerequisite lists: [F02.1](../01-input-inventory/README.md) and
[F02.2](../02-fetch-verifier/README.md).

## Outcome

Every future ABI fixture or generated protocol artifact identifies exactly which verified input made it.

## Starting points

- [input inventory](../01-input-inventory/README.md)
- [fetch verifier](../02-fetch-verifier/README.md)
- [graphics workflow](../../../../workflow.md)

## Checklist

- [ ] Define a short provenance record with manifest revision, input entry IDs, source license,
  fetch/build command, generator version, and generated-output hash.
- [ ] Require the record for new ABI fixtures and generated protocol outputs; distinguish copied
  upstream bytes from maintained hand-written adapters.
- [ ] Add a deterministic test that rejects a missing, unknown, or stale manifest reference.
- [ ] Record the fresh-fetch output, provenance test count, and any unavailable upstream input in a
  receipt before completing this child.

## Verification

- A fixture/provenance sample resolves only to the verified manifest revision and fails after a
  deliberately changed input ID or digest.
- The parent F02 acceptance runs a clean fetch plus the provenance check before its own receipt.
