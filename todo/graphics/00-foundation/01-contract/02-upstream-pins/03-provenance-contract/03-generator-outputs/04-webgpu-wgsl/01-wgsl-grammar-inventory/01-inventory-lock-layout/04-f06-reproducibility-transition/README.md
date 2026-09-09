# F02.3.3.4.1.1.4 — Prepare reproducibility consumers for the inventory revision

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.3.3.4.1.1.4
Depends: F02.1, F06.3.3, F02.3.3.4.1.1.1
Evidence: pending

Prerequisite lists: [the composite-lock contract](../01-shared-layout-loader/README.md) and
[F06.3.3](../../../../../../../../02-reproducibility/03-file-layout/03-generation-and-checker/03-reproducibility-proof/README.md).

## Outcome

The F06 chunk specification and metadata can express an `inventory_sha256` that binds the canonical
lock, while the committed fixture stays on its current schema until the atomic cutover.

## Starting points

- [chunk generator](../../../../../../../../../../../scripts/graphics_chunker.py)
- [chunk-generator tests](../../../../../../../../../../../scripts/test_graphics_chunker.py)
- [reproducibility proof](../../../../../../../../../../../scripts/test_graphics_reproducibility.py)

## Checklist

- [ ] Version chunk specifications and metadata so their source revision denotes the canonical
  inventory lock identity.
- [ ] Make the chunk generator reject a stale lock identity without learning inventory composition.
- [ ] Add hermetic v1/v2 fixture tests for stale locks and reproducible chunk output.
- [ ] Leave the checked-in F06 fixture and generated metadata on their current identity until the
  atomic cutover child; run focused tests and preserve the 180-line limit.

## Verification

- A v2 chunk specification does not accept a manifest-only field or mismatched lock bytes.
- Repeated generation remains byte-identical for each supported specification version.
