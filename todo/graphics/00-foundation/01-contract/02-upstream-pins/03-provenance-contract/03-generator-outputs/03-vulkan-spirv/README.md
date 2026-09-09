# F02.3.3.3 — Bind Vulkan and SPIR-V generator records

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.3.3.3
Depends: F02.1, F02.2, F02.3.1
Evidence: pending

Prerequisite lists: [F02.3.1](../../01-provenance-record/README.md) and
[F02.2](../../../02-fetch-verifier/README.md).

## Outcome

Fixture-only future Vulkan registry and SPIR-V grammar outputs each name their own reviewed input,
without claiming a Vulkan guest, Venus runtime, or generated protocol implementation.

## Starting points

- [input manifest](../../../01-input-inventory/manifest.toml)
- [record contract](../../01-provenance-record/README.md)
- [Venus foundations](../../../../../../../../research/venus-foundations.md)

## Checklist

- [ ] Bind a Vulkan registry sample only to `vulkan-registry` and a SPIR-V sample only to `spirv-core-grammar`.
- [ ] Record each exact manifest ID, digest, license, command, generator name/version, and output hash.
- [ ] Reject a cross-family digest, `vk-gl-cts-api-version`, or runtime source as a generator-input substitute.
- [ ] Keep both samples fixture-only and avoid vendoring registry or grammar bytes.

## Verification

- Each generated record resolves only to its reviewed registry or grammar input.
- Wrong generator version, input identity, or output hash fails deterministically offline.
