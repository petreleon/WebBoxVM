# F02.4.1 — Audit OpenGL 4.6 core normative and CTS inputs

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.4.1
Depends: F02.1, F02.2, F03.1
Evidence: pending

Prerequisite lists: [F02.2](../../02-fetch-verifier/README.md) and
[F03.1](../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

The proposed `opengl-46-core-spec` and `opengl-cts-manifest` inputs are verified as immutable,
license-compatible, fetchable materials that cover the stated core profile and complete selector set.

## Starting points

- [required OpenGL IDs](../../../03-feature-matrix/01-profile-scope/source_requirements.json)
- [fetch policy](../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [current isolated Piglit case](../../01-input-inventory/inputs/part-0002.toml)

## Checklist

- [ ] Locate a version-specific OpenGL 4.6 core normative source and record its immutable identity.
- [ ] Locate a complete committed GL CTS selector/catalog source; reject a single Piglit case.
- [ ] Verify URL host, revision, license, byte size, and digest against the F02.2 policy.
- [ ] Add hermetic positive and hostile candidate tests without changing the shared inventory.
- [ ] Record the candidate rationale, limits, and exact selectors for F02.4.4 admission.

## Verification

- The audit has at least one accepted or concretely rejected candidate for each required ID.
- No compatibility-profile, optional-extension, or GL 3.0-only artifact is called OpenGL 4.6 core
  coverage.
