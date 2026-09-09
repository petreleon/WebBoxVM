# F02.3.1 — Define the provenance record contract

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.3.1
Depends: F02.1, F02.2
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.1](../../01-input-inventory/README.md) and
[F02.2](../../02-fetch-verifier/README.md).

## Outcome

A small, versioned provenance record binds a reviewed artifact to the exact verified inventory input.

## Starting points

- [input manifest](../../01-input-inventory/manifest.toml)
- [fetch contract](../../02-fetch-verifier/01-fetch-contract/README.md)
- [graphics workflow](../../../../../workflow.md)

## Checklist

- [x] Define required record fields: inventory revision, input IDs/digests, license, command,
  generator identity/version, artifact kind, and output hash.
- [x] Model `handwritten`, `copied-upstream`, and `generated` artifacts without calling maintained
  adapters generated when no generator produced them.
- [x] Add a deterministic parser/validator that resolves only F02.1 input IDs and digest identities.
- [x] Test missing, unknown, and stale inventory or input references without network access.

## Verification

- A valid sample resolves to the reviewed inventory-lock SHA-256 and input identity.
- A changed inventory-lock SHA-256, ID, or digest fails before the record can be accepted.

## Contract

`provenance_record.py` reads JSON sidecars and the F02.1 TOML inventory with standard-library
parsers only. A schema-v2 record has an exact schema: raw `inventory.lock` SHA-256; sorted input
ID/SHA-256/license references; command; generator name/version; artifact kind/path; and output
SHA-256. `generated` requires a generator, `copied-upstream` must have one input whose digest equals
its output digest, and `handwritten` must use `generator: none` without reusing an input's byte
identity.

The canonical authority is [F02.1's inventory lock](../../01-input-inventory/inventory.lock): its
exact raw bytes—not an upstream commit, root manifest fragment, or cache result—provide
`inventory_sha256`. Active consumers load only schema-v2 inventory closures; legacy schema-v1
sidecars are parsed only far enough to reject their schema mismatch. This leaf validates record
schema only and creates no real ABI artifact binding or generator-output binding.

Run the hermetic suite with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/01-provenance-record/provenance_record_test.py
```

This leaf defines no actual ABI fixture or generator-output binding; F02.3.2 and F02.3.3 own those
records, while F02.3.4 verifies them against the fresh F02.2 cache.
