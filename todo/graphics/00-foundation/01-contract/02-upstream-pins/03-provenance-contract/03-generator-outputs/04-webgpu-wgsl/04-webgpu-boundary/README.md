# F02.3.3.4.4 — Hold the WebGPU generator-input boundary

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.3.3.4.4
Depends: F02.1, F02.2, F02.3.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.1](../../../../01-input-inventory/README.md),
[F02.2](../../../../02-fetch-verifier/README.md), and
[F02.3.1](../../../01-provenance-record/README.md).

## Outcome

WebGPU reference documents remain ineligible as generator inputs until an independently reviewed,
immutable WebGPU source definition is available; no fixture or runtime claim fills that gap.

## Starting points

- [current immutable inventory](../../../../01-input-inventory/manifest.toml)
- [F02.3.1 validator](../../../01-provenance-record/README.md)
- [GPUWeb license at the current pin](https://github.com/gpuweb/gpuweb/blob/e0aff163a37eb3633ffd612e2a943ceb6196d6af/LICENSE.md)

## Checklist

- [ ] Audit the inventory for a WebGPU source definition distinct from `webgpu-spec`.
- [ ] Reject `webgpu-spec`, `wgsl-spec`, and any WGSL grammar input as a WebGPU generator input.
- [ ] Record absence of an eligible source as a blocker rather than inventing an API record.
- [ ] Preserve all WebGPU reference material outside generated code and upstream payloads.

## Verification

- A reference-only or WGSL-only input cannot validate as a WebGPU generator input offline.
- This child stays open until the immutable inventory contains an honest WebGPU generator input.

## Blocker probe

`webgpu_generator_boundary.py` reads only the local F02.1 manifest. It accepts a blocked result
only while no input has the exact `future WebGPU generator input` designation, and it rejects every
`wgsl` input—including a temporary grammar candidate—as a WebGPU generator input. Run the hermetic
suite with `PYTHONDONTWRITEBYTECODE=1 python3 webgpu_generator_boundary_test.py` from this folder.
