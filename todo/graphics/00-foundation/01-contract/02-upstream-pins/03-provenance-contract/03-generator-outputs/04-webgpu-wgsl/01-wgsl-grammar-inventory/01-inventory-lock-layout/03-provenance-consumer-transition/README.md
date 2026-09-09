# F02.3.3.4.1.1.3 — Prepare provenance consumers for the inventory revision

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.3.3.4.1.1.3
Depends: F02.1, F02.2, F02.3.1, F02.3.3.4.1.1.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [the composite-lock contract](../01-shared-layout-loader/README.md) and
[F02.3.1](../../../../../01-provenance-record/README.md).

## Outcome

Before the atomic cutover, the provenance contract could validate an explicit schema-v2
`inventory_sha256` against the shared layout while retaining the committed schema-v1 sidecars.

## Starting points

- [record validator](../../../../../01-provenance-record/provenance_record.py)
- [ABI sidecars](../../../../../02-abi-fixtures/README.md)
- [WebGPU boundary probe](../../../04-webgpu-boundary/webgpu_generator_boundary.py)

## Checklist

- [x] Version the provenance-record contract so schema-v2 records name the canonical inventory
  revision rather than a raw one-file manifest hash.
- [x] Resolve ABI, Venus, GL/GLES, Vulkan/SPIR-V, and WebGPU-boundary inputs through the shared
  layout loader.
- [x] Add v1/v2 hermetic record fixtures that reject stale lock identities, legacy fields under v2,
  and undeclared component inputs.
- [x] Do not alter checked-in sidecars, artifact bytes, or source identities until the atomic
  cutover child; run every affected focused offline suite.

## Verification

- A schema-v2 record cannot accept a schema-v1 manifest hash or a stale lock identity.
- Existing committed schema-v1 sidecars retained their historical offline validation before cutover.
