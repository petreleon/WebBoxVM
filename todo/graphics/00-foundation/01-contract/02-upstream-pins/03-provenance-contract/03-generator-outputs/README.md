# F02.3.3 — Bind generated protocol outputs

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.3.3
Depends: F02.1, F02.2, F02.3.1
Evidence: pending

Prerequisite lists: [F02.3.1](../01-provenance-record/README.md) and
[F02.2](../../02-fetch-verifier/README.md).

## Outcome

Every future protocol generator records the immutable registry input and exact generated output it used.

## Starting points

- [generator-role inventory](../../01-input-inventory/manifest.toml)
- [VirGL2 boundary](../../../../../../../research/virgl2-capset.md)
- [Venus foundations](../../../../../../../research/venus-foundations.md)

## Checklist

- [ ] Define record samples for Venus, GL/GLES, Vulkan/SPIR-V, and WebGPU/WGSL generator families.
- [ ] Require each generated output to name its manifest input ID/digest, command, generator version,
  license, and output hash.
- [ ] Reject a generated record that substitutes a spec/reference input for its declared generator input.
- [ ] Keep ungenerated reference material distinct from generated code and avoid vendoring source bytes.

## Verification

- A generated-output sample resolves only to the reviewed registry or grammar input.
- A wrong generator version, input digest, or output hash fails deterministically offline.
