# F02.3.3.4.4.1 — Admit a reviewed WebGPU WebIDL source

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.3.3.4.4.1
Depends: F02.1, F02.2, F02.3.1
Evidence: pending

Prerequisite lists: [F02.1](../../../../../01-input-inventory/README.md),
[F02.2](../../../../../02-fetch-verifier/README.md), and the
[source audit](../../../../../../../../../../research/webgpu-generator-source-audit.md).

## Outcome

The immutable inventory names a distinct, verified GPUWeb WebIDL input suitable
only for future DOM-binding/schema generation; `webgpu-spec` remains a semantic
reference and no browser, guest, rendering, or compatibility claim is made.

## Starting points

- [current inventory root](../../../../../01-input-inventory/manifest.toml)
- [source records](../../../../../01-input-inventory/inputs/part-0001.toml)
- [F02.2 live fetch verification](../../../../../02-fetch-verifier/03-live-inventory/README.md)
- [candidate audit](../../../../../../../../../../research/webgpu-generator-source-audit.md)

## Checklist

- [ ] Verify the immutable WebIDL URL, revision, byte count, SHA-256, and W3C license signal.
- [ ] Add a distinct `webgpu-idl` family and source without reclassifying `webgpu-spec`.
- [ ] Regenerate the canonical F02.1 lock and prove structural validation rejects malformed input.
- [ ] Fetch the new source through F02.2 and record a local, hash-verified live-inventory result.

## Verification

- The inventory keeps one semantic `webgpu` reference and one distinct `webgpu-idl` generator input.
- A clean F02.2 cache fetch verifies the exact WebIDL bytes and fails closed on an altered digest.

## Scope limit

This task admits provenance metadata only. It does not generate a binding, use WebIDL at runtime,
add JavaScript/WebGPU behavior, expose a guest API, render a frame, or establish any graphics profile.
