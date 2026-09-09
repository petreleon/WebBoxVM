# F02.3.3.4 — Establish the WebGPU/WGSL generator-input boundary

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.3.3.4
Depends: F02.1, F02.2, F02.3.1
Evidence: pending

Prerequisite lists: [F02.3.1](../../01-provenance-record/README.md) and
[F02.2](../../../02-fetch-verifier/README.md).

## Outcome

WebGPU and WGSL references are kept out of generated-output records until F02.1 contains a reviewed
input explicitly designated for generator use; no WebGPU/WGSL generation or runtime claim is made.

## Starting points

- [input manifest](../../../01-input-inventory/manifest.toml)
- [record contract](../../01-provenance-record/README.md)
- [WebGPU/WGSL roles](../../../01-input-inventory/README.md)

## Checklist

- [ ] Inspect the manifest roles for an explicit WebGPU or WGSL generator input before adding a record.
- [ ] Reject `webgpu-spec` and `wgsl-spec` as generated-output inputs while their roles say no generated code.
- [ ] Record the missing designated input as a blocker rather than inventing a generator-output sample.
- [ ] Preserve WebGPU/WGSL reference material outside generated code and do not vendor source bytes.

## Verification

- A reference-only WebGPU/WGSL input cannot be accepted as a generator input offline.
- This child remains open unless the immutable inventory gains an honest reviewed generator input.
