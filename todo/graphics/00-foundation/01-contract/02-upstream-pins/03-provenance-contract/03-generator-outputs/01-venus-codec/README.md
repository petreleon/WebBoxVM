# F02.3.3.1 — Bind a Venus codec generator record

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.3.3.1
Depends: F02.1, F02.2, F02.3.1
Evidence: [receipt](evidence.md)

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

- [x] Select only `venus-protocol-registry` as the registry-generator input for the sample.
- [x] Record its exact manifest ID, digest, license, command, generator name/version, and output hash.
- [x] Keep the sample output fixture-only; do not vendor registry bytes or claim a Venus implementation.
- [x] Reject a changed generator version, input digest, or output hash through the F02.3.1 validator.

## Verification

- The generated record resolves only to the pinned Venus protocol registry identity.
- A reference or runtime source substituted for that input fails deterministically offline.

## Fixture boundary

`venus_fixture_generator.py` emits a four-line provenance marker from the registry identity only;
it does not read, vendor, parse, or implement the upstream Venus protocol. The sidecar is checked
against the F02.1 manifest and the artifact's actual bytes by `venus_record.py`. Run the hermetic
suite with `PYTHONDONTWRITEBYTECODE=1 python3 venus_record_test.py` from this folder.
