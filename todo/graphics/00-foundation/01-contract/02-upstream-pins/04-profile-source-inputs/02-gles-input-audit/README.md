# F02.4.2 — Audit GLES 3.2 normative and CTS inputs

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.4.2
Depends: F02.1, F02.2, F03.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.2](../../02-fetch-verifier/README.md) and
[F03.1](../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

The audit accepts the GLES 3.2 normative PDF and records the CTS descriptor as an immutable but
currently rejected compound input. Its four core selector files and twelve descriptor configurations
are bound exactly, while the optional GLES extension selector stays explicitly excluded.

## Starting points

- [required GLES IDs](../../../03-feature-matrix/01-profile-scope/source_requirements.json)
- [ES shading-language source](../../01-input-inventory/inputs/part-0001.toml)
- [fetch policy](../../02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [x] Locate a version-specific GLES 3.2 normative source and record its immutable identity.
- [x] Locate a complete committed ES CTS selector/catalog source; reject a desktop GL surrogate.
- [x] Verify URL host, revision, license, byte size, and digest against the F02.2 policy.
- [x] Add hermetic positive and hostile candidate tests without changing the shared inventory.
- [x] Record the candidate rationale, limits, and exact selectors for F02.4.4 admission.
- [ ] Publish the verified audit commit and record the matching remote SHA.

## Verification

- The audit has at least one accepted or concretely rejected candidate for each required ID.
- Neither desktop compatibility behavior nor optional extensions is silently included in GLES 3.2.

## Limits

This is source provenance and descriptor analysis only. It does not admit an inventory input, parse
the descriptor from a refreshed cache, build or run CTS, establish conformance, expose a guest API,
or measure browser performance. F02.4.4 must implement the atomic admission and live parsing path.
