# F02.3.3.2 — Bind a GL/GLES registry generator record

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.3.3.2
Depends: F02.1, F02.2, F02.3.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.3.1](../../01-provenance-record/README.md) and
[F02.2](../../../02-fetch-verifier/README.md).

## Outcome

One fixture-only future GL/GLES registry output has a generated provenance sidecar bound to the
reviewed registry, while GLSL and ESSL specifications stay reference material.

## Starting points

- [input manifest](../../../01-input-inventory/manifest.toml)
- [record contract](../../01-provenance-record/README.md)
- [GL/GLES registry role](../../../01-input-inventory/README.md)

## Checklist

- [x] Select only `opengl-gles-registry` as the registry-generator input for the sample.
- [x] Record its exact manifest ID, digest, license, command, generator name/version, and output hash.
- [x] Reject `glsl-460-spec` or `essl-320-spec` when offered as the generated-output input.
- [x] Keep reference specifications and any fixture output distinct; do not vendor upstream bytes.

## Verification

- The generated record resolves only to the pinned GL/GLES registry identity.
- A semantic specification, wrong version, digest, or output hash fails deterministically offline.
