# F02.4.3 — Audit Vulkan 1.4 normative and must-pass inputs

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.4.3
Depends: F02.1, F02.2, F03.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.2](../../02-fetch-verifier/README.md) and
[F03.1](../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

The audit records immutable source roots for both required IDs and rejects both for admission now.
`vulkan-14-spec` is an AsciiDoc root with an unresolved generated/transitive/core-configuration
closure. `vulkan-cts-mustpass` is an upstream default root with 98 member lists, oversize members,
and explicit WSI/extension scope outside the frozen Vulkan core profile. This is provenance and
boundary analysis, not compatibility or execution evidence.

## Starting points

- [required Vulkan IDs](../../../03-feature-matrix/01-profile-scope/source_requirements.json)
- [Vulkan and SPIR-V inputs](../../01-input-inventory/inputs/part-0002.toml)
- [fetch policy](../../02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [x] Locate a version-specific Vulkan 1.4 normative source and record its immutable identity.
- [x] Locate a complete committed Vulkan CTS must-pass selector/catalog; reject one API-version case.
- [x] Verify URL host, revision, license, byte size, and digest against the F02.2 policy.
- [x] Add hermetic positive and hostile candidate tests without changing the shared inventory.
- [x] Record the candidate rationale, limits, and exact selectors for F02.4.4 admission.
- [x] Publish the verified audit commit and record the matching remote SHA.

## Verification

- The audit has at least one accepted or concretely rejected candidate for each required ID.
- Promoted-extension provenance, WSI, and portability subset stay explicit separate scope records.

## Limits

The 73-directive specification transcript does not bind macro or `ifdef` resolution, generated
`specattribs.adoc`, or transitive include identities. The VCTS root-byte record does not bind the
content, hashes, recursive dependencies, or core-only selection of its 98 referenced files. Both
candidates therefore remain rejected. This work neither changes the shared inventory nor runs CTS,
exposes a guest API, runs a browser workload, establishes conformance, or measures performance.
