# F02.4.2 — Audit GLES 3.2 normative and CTS inputs

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.4.2
Depends: F02.1, F02.2, F03.1
Evidence: pending

Prerequisite lists: [F02.2](../../02-fetch-verifier/README.md) and
[F03.1](../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

The proposed `gles-32-spec` and `gles-cts-manifest` inputs are verified as immutable materials for
GLES 3.2 and its complete conformance selector set, separate from desktop compatibility semantics.

## Starting points

- [required GLES IDs](../../../03-feature-matrix/01-profile-scope/source_requirements.json)
- [ES shading-language source](../../01-input-inventory/inputs/part-0001.toml)
- [fetch policy](../../02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [ ] Locate a version-specific GLES 3.2 normative source and record its immutable identity.
- [ ] Locate a complete committed ES CTS selector/catalog source; reject a desktop GL surrogate.
- [ ] Verify URL host, revision, license, byte size, and digest against the F02.2 policy.
- [ ] Add hermetic positive and hostile candidate tests without changing the shared inventory.
- [ ] Record the candidate rationale, limits, and exact selectors for F02.4.4 admission.

## Verification

- The audit has at least one accepted or concretely rejected candidate for each required ID.
- Neither desktop compatibility behavior nor optional extensions is silently included in GLES 3.2.
