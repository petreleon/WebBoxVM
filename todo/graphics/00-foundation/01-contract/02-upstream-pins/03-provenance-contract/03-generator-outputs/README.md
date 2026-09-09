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

- [x] [F02.3.3.1 — Bind a Venus codec generator record](01-venus-codec/README.md)
- [x] [F02.3.3.2 — Bind a GL/GLES registry generator record](02-gl-gles-registry/README.md)
- [ ] [F02.3.3.3 — Bind Vulkan and SPIR-V generator records](03-vulkan-spirv/README.md)
- [ ] [F02.3.3.4 — Establish the WebGPU/WGSL generator-input boundary](04-webgpu-wgsl/README.md)

## Verification

- A generated-output sample resolves only to the reviewed registry or grammar input.
- A wrong generator version, input digest, or output hash fails deterministically offline.

## Split rationale

Venus, GL/GLES, Vulkan/SPIR-V, and WebGPU/WGSL have separate immutable inputs and distinct
reference-versus-generator boundaries. Each child can therefore add one honest record family and
its negative checks without allowing a passing record for a reference-only source.
