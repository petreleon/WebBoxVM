# F02.3.3.4 — Establish the WebGPU/WGSL generator-input boundary

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.3.3.4
Depends: F02.1, F02.2, F02.3.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.3.1](../../01-provenance-record/README.md) and
[F02.2](../../../02-fetch-verifier/README.md).

## Outcome

The provenance foundation distinguishes the locked WGSL grammar record from the narrow reviewed
`webgpu-idl` binding/interop input. WebGPU and WGSL semantic references remain ineligible;
no WebGPU/WGSL generation or runtime claim is made.

## Starting points

- [input manifest](../../../01-input-inventory/manifest.toml)
- [record contract](../../01-provenance-record/README.md)
- [WebGPU/WGSL roles](../../../01-input-inventory/README.md)

## Checklist

- [x] [F02.3.3.4.1 — Lock and verify a WGSL grammar input](01-wgsl-grammar-inventory/README.md)
- [x] [F02.3.3.4.2 — Renew inventory-bound records](02-record-renewal/README.md)
- [x] [F02.3.3.4.3 — Bind a WGSL grammar generator record](03-wgsl-generator-record/README.md)
- [x] [F02.3.3.4.4 — Close the WebGPU generator-input boundary](04-webgpu-boundary/README.md)

## Verification

- A reference-only WebGPU/WGSL input cannot be accepted as a generator input offline.
- Only the reviewed `webgpu-idl` identity passes its separate WebGPU boundary; the WGSL grammar
  record remains scoped to its own fixture.

## Split rationale

WGSL syntax grammar is locked and re-fetched before its distinct record-renewal pass, so its fixture
can be honest without treating `wgsl-spec` as generator material. The separately admitted
WebIDL record is checked by exact identity rather than silently treating `webgpu-spec` as a
generator input. Neither provenance record implements a WebGPU/WGSL runtime.
