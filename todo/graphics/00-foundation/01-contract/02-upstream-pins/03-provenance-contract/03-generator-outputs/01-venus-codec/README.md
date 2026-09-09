# F02.3.3.1 — Bind a Venus codec generator record

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.3.3.1
Depends: F02.1, F02.2, F02.3.1
Evidence: pending

Prerequisite lists: [F02.3.1](../../01-provenance-record/README.md) and
[F02.2](../../../02-fetch-verifier/README.md).

## Outcome

One fixture-only future Venus codec output has an honest generated provenance sidecar bound to the
reviewed Venus protocol registry, without suggesting that a Venus guest or runtime exists.

## Starting points

- [input manifest](../../../01-input-inventory/manifest.toml)
- [record contract](../../01-provenance-record/README.md)
- [Venus foundations](../../../../../../../../research/venus-foundations.md)

## Checklist

- [ ] Select only `venus-protocol-registry` as the registry-generator input for the sample.
- [ ] Record its exact manifest ID, digest, license, command, generator name/version, and output hash.
- [ ] Keep the sample output fixture-only; do not vendor registry bytes or claim a Venus implementation.
- [ ] Reject a changed generator version, input digest, or output hash through the F02.3.1 validator.

## Verification

- The generated record resolves only to the pinned Venus protocol registry identity.
- A reference or runtime source substituted for that input fails deterministically offline.
