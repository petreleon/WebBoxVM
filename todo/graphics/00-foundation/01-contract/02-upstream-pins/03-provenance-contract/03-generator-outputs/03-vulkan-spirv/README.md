# F02.3.3.3 — Bind Vulkan and SPIR-V generator records

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.3.3.3
Depends: F02.1, F02.2, F02.3.1
Evidence: [receipt](evidence.md)

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

- [x] Bind a Vulkan registry sample only to `vulkan-registry` and a SPIR-V sample only to `spirv-core-grammar`.
- [x] Record each exact manifest ID, digest, license, command, generator name/version, and output hash.
- [x] Reject a cross-family digest, `vk-gl-cts-api-version`, or runtime source as a generator-input substitute.
- [x] Keep both samples fixture-only and avoid vendoring registry or grammar bytes.

## Verification

- Each generated record resolves only to its reviewed registry or grammar input.
- Wrong generator version, input identity, or output hash fails deterministically offline.

## Fixture boundary

The checked generator emits only deterministic, local provenance markers from reviewed identities; it
does not read or copy registry/grammar payloads and does not generate Vulkan, SPIR-V, Venus, or guest
protocol code. The two committed outputs are test fixtures, not an implementation claim.

Run the focused suite with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/03-generator-outputs/03-vulkan-spirv/validate_vulkan_spirv.py
```
