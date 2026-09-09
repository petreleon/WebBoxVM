# F02.3.3.4.4.3 — Bind a WebGPU WebIDL generator record

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.3.3.4.4.3
Depends: F02.3.3.4.4.1, F02.3.3.4.4.2, F02.3.1
Evidence: pending

Prerequisite lists: [WebIDL source admission](../01-webidl-source-admission/README.md),
[renewed records](../02-lock-record-renewal/README.md), and the
[provenance contract](../../../../01-provenance-record/README.md).

## Outcome

A minimal, reproducible provenance marker binds only the reviewed WebIDL source
and its exact inventory revision; it proves generator-input accounting, not an
implemented WebGPU binding or a graphics path.

## Starting points

- [WebIDL input record](../../../../../01-input-inventory/inputs/part-0002.toml)
- [WGSL marker precedent](../../03-wgsl-generator-record/README.md)
- [provenance validator](../../../../01-provenance-record/provenance_record.py)

## Checklist

- [ ] Define a small marker and sidecar that declare the WebIDL-only input and its limitation.
- [ ] Generate and validate them deterministically from the checked inventory identity.
- [ ] Add positive, stale-identity, wrong-input, altered-byte, and scope-tampering tests.
- [ ] Keep upstream bytes, a parser, runtime code, and host/guest behavior out of the artifact.

## Verification

- The marker validates only for the exact `webgpu-idl` input, lock, license, role, and output hash.
- Any semantic reference, WGSL input, CTS entry, or changed scope limitation is rejected offline.

## Scope limit

The marker is not a binding generator, parser, compiler, API implementation, guest device,
browser renderer, compatibility proof, or performance result.
