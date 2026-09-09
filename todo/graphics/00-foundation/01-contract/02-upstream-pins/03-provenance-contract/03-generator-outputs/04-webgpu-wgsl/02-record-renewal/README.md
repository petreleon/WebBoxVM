# F02.3.3.4.2 — Renew lock-bound records and reproducibility proof

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.3.3.4.2
Depends: F02.3.1, F02.3.3.4.1
Evidence: pending

Prerequisite lists: [F02.3.1](../../../01-provenance-record/README.md) and
[the WGSL grammar lock](../01-wgsl-grammar-inventory/README.md).

## Outcome

Every existing F02.3 sidecar and the F06 reproducibility fixture are renewed to the new canonical
F02.1 `inventory.lock` SHA-256 without changing any declared source identity or local artifact byte.

## Starting points

- [provenance record contract](../../../01-provenance-record/README.md)
- [ABI sidecars](../../../02-abi-fixtures/README.md)
- [existing generated fixture parent](../../README.md)
- [F06 reproducibility fixture](../../../../../../02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/README.md)

## Checklist

- [ ] Recompute the canonical raw `inventory.lock` SHA-256 after F02.3.3.4.1's accepted grammar
  change.
- [ ] Renew F02.3.1 test fixtures, ABI sidecars, Venus/GL/Vulkan/SPIR-V fixture sidecars, and the
  F06 input/output hashes to that one lock identity.
- [ ] Prove their input IDs, digests, licenses, commands, generator versions, and artifact hashes are
  otherwise unchanged.
- [ ] Run every affected focused offline suite and report exact nonzero counts.

## Verification

- The old raw lock SHA-256 fails every renewed record and the F06 fixture.
- No renewed record converts reference material into generated code or changes graphics behavior.
