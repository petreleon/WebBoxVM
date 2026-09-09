# F02.3.3.4.3 — Bind a WGSL grammar generator record

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.3.3.4.3
Depends: F02.3.1, F02.3.3.4.1, F02.3.3.4.2
Evidence: pending

Prerequisite lists: [F02.3.1](../../../01-provenance-record/README.md),
[the grammar lock](../01-wgsl-grammar-inventory/README.md), and
[record renewal](../02-record-renewal/README.md).

## Outcome

A fixture-only WGSL grammar output records only the accepted grammar input and its reviewed local
generator, without claiming a WGSL compiler, WebGPU API binding, or guest rendering behavior.

## Starting points

- [accepted grammar lock](../01-wgsl-grammar-inventory/README.md)
- [record-renewal prerequisite](../02-record-renewal/README.md)
- [current WGSL reference role](../../../../01-input-inventory/manifest.toml)

## Checklist

- [ ] Bind exactly one generated fixture record to the new WGSL grammar input and its license.
- [ ] Record the generator command/version, output hash, and grammar-dialect limitation.
- [ ] Reject the current `wgsl-spec`, `webgpu-spec`, cross-family sources, wrong generator version,
  input digest, and output hash offline.
- [ ] Keep the fixture free of upstream grammar bytes and separate from WebGPU API generation.

## Verification

- The fixture reproduces from its declared local generator and manifest identity only.
- It does not establish WebGPU support, WGSL compilation, or a guest-visible API.
