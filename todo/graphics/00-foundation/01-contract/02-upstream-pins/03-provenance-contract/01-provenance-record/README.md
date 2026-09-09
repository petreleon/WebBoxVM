# F02.3.1 — Define the provenance record contract

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.3.1
Depends: F02.1, F02.2
Evidence: pending

Prerequisite lists: [F02.1](../../01-input-inventory/README.md) and
[F02.2](../../02-fetch-verifier/README.md).

## Outcome

A small, versioned provenance record binds a reviewed artifact to the exact verified manifest input.

## Starting points

- [input manifest](../../01-input-inventory/manifest.toml)
- [fetch contract](../../02-fetch-verifier/01-fetch-contract/README.md)
- [graphics workflow](../../../../../workflow.md)

## Checklist

- [ ] Define required record fields: manifest revision, input IDs/digests, license, command,
  generator identity/version, artifact kind, and output hash.
- [ ] Model `handwritten`, `copied-upstream`, and `generated` artifacts without calling maintained
  adapters generated when no generator produced them.
- [ ] Add a deterministic parser/validator that resolves only F02.1 input IDs and digest identities.
- [ ] Test missing, unknown, and stale manifest or input references without network access.

## Verification

- A valid sample resolves to the reviewed manifest SHA-256 and input identity.
- A changed manifest SHA-256, ID, or digest fails before the record can be accepted.
