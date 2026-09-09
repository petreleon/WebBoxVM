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

- [x] [F02.3.3.4.1 — Lock and verify a WGSL grammar input](01-wgsl-grammar-inventory/README.md)
- [ ] [F02.3.3.4.2 — Renew inventory-bound records](02-record-renewal/README.md)
- [ ] [F02.3.3.4.3 — Bind a WGSL grammar generator record](03-wgsl-generator-record/README.md)
- [ ] [F02.3.3.4.4 — Hold the WebGPU generator-input boundary](04-webgpu-boundary/README.md)

## Verification

- A reference-only WebGPU/WGSL input cannot be accepted as a generator input offline.
- This parent remains open unless the immutable inventory gains an honest reviewed WebGPU generator input.

## Split rationale

WGSL has a candidate syntax grammar that must first be locked and re-fetched; that changes the raw
F02.1 inventory-lock digest and therefore needs a distinct record-renewal pass before any WGSL fixture can
be honest. No equivalent immutable WebGPU generator input is currently identified, so its boundary
must remain independently open rather than silently treating `webgpu-spec` as generator material.
