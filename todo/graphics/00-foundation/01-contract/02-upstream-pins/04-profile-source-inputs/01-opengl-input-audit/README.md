# F02.4.1 — Audit OpenGL 4.6 core normative and CTS inputs

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.4.1
Depends: F02.1, F02.2, F03.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.2](../../02-fetch-verifier/README.md) and
[F03.1](../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

The audit records exact immutable identities and stated license terms for an OpenGL 4.6 core PDF and
a flat released GL 4.6 selector. It is source provenance, not proof of API coverage or execution.

## Starting points

- [required OpenGL IDs](../../../03-feature-matrix/01-profile-scope/source_requirements.json)
- [fetch policy](../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [current isolated Piglit case](../../01-input-inventory/inputs/part-0002.toml)

## Checklist

- [x] Locate a version-specific OpenGL 4.6 core normative source and record its immutable identity.
- [x] Locate a complete committed GL CTS selector/catalog source; reject a single Piglit case.
- [x] Verify URL host, revision, license, byte size, and digest against the F02.2 policy.
- [x] Add hermetic positive and hostile candidate tests without changing the shared inventory.
- [x] Record the candidate rationale, limits, and exact selectors for F02.4.4 admission.

## Verification

- The audit has at least one accepted or concretely rejected candidate for each required ID.
- No compatibility-profile, optional-extension, or GL 3.0-only artifact is called OpenGL 4.6 core
  coverage.

## Limits

This receipt accepts provenance for a released GL 4.6 selector only. It does not pin the CTS build
closure, execute CTS, establish conformance, expose a guest API, or measure browser performance.
