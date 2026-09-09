# F02.4.3 — Audit Vulkan 1.4 normative and must-pass inputs

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.4.3
Depends: F02.1, F02.2, F03.1
Evidence: pending

Prerequisite lists: [F02.2](../../02-fetch-verifier/README.md) and
[F03.1](../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

The proposed `vulkan-14-spec` and `vulkan-cts-mustpass` inputs are verified as immutable Vulkan 1.4
normative and complete must-pass materials, not one version-check source file.

## Starting points

- [required Vulkan IDs](../../../03-feature-matrix/01-profile-scope/source_requirements.json)
- [Vulkan and SPIR-V inputs](../../01-input-inventory/inputs/part-0002.toml)
- [fetch policy](../../02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [ ] Locate a version-specific Vulkan 1.4 normative source and record its immutable identity.
- [ ] Locate a complete committed Vulkan CTS must-pass selector/catalog; reject one API-version case.
- [ ] Verify URL host, revision, license, byte size, and digest against the F02.2 policy.
- [ ] Add hermetic positive and hostile candidate tests without changing the shared inventory.
- [ ] Record the candidate rationale, limits, and exact selectors for F02.4.4 admission.

## Verification

- The audit has at least one accepted or concretely rejected candidate for each required ID.
- Promoted-extension provenance, WSI, and portability subset stay explicit separate scope records.
