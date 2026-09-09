# F02.3 — Bind ABI fixtures and generators to the immutable inventory

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F02.3
Depends: F02.1, F02.2
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.1](../01-input-inventory/README.md) and
[F02.2](../02-fetch-verifier/README.md).

## Outcome

Every future ABI fixture or generated protocol artifact identifies exactly which verified input made it.

## Starting points

- [input inventory](../01-input-inventory/README.md)
- [fetch verifier](../02-fetch-verifier/README.md)
- [graphics workflow](../../../../workflow.md)

## Checklist

- [x] [F02.3.1 — Define the provenance record contract](01-provenance-record/README.md)
- [x] [F02.3.2 — Bind ABI fixtures and adapters](02-abi-fixtures/README.md)
- [x] [F02.3.3 — Bind generated protocol outputs](03-generator-outputs/README.md)
- [x] [F02.3.4 — Verify provenance closure](04-provenance-validation/README.md)

## Verification

- A fixture/provenance sample resolves only to the verified inventory-lock identity and fails after a
  deliberately changed input ID or digest.
- The parent F02 acceptance runs a clean fetch plus the provenance check before its own receipt.

## Split rationale

The shared provenance format and validator, ABI-facing hand-written or copied adapters, and future
generator output families have independent owners and change cadence. They are split before code so
one record format can be reviewed independently, while the two consumer families and final live
verification remain separate, testable commitments.
