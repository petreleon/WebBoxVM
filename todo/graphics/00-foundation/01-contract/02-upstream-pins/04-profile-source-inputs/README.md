# F02.4 — Pin target-profile normative sources and conformance manifests

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F02.4
Depends: F02.1, F02.2, F03.1
Evidence: pending

Prerequisite lists: [F02.1](../01-input-inventory/README.md),
[F02.2](../02-fetch-verifier/README.md), and [F03.1](../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

Each final target receives a reviewed immutable normative source and complete pinned conformance
selector input, without treating an isolated Piglit or API-version case as a full suite.

## Starting points

- [F03.1 source gate](../../03-feature-matrix/01-profile-scope/source_requirements.json)
- [F02 inventory schema](../01-input-inventory/inventory_layout.py)
- [F02 fetch contract](../02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [x] [F02.4.1 — Audit OpenGL 4.6 core normative and CTS inputs](01-opengl-input-audit/README.md)
- [x] [F02.4.2 — Audit GLES 3.2 normative and CTS inputs](02-gles-input-audit/README.md)
- [x] [F02.4.3 — Audit Vulkan 1.4 normative and must-pass inputs](03-vulkan-input-audit/README.md)
- [ ] [F02.4.4 — Atomically admit sources and renew lock consumers](04-atomic-admission/README.md)
- [ ] Re-fetch the complete revised inventory, verify provenance closure and the F03.1 source gate,
  then attach the aggregate receipt.

## Verification

- Every admitted input has an immutable reviewed URL, revision, digest, byte count, license, and
  fetch-policy proof; a generated or partial selector cannot stand in for a complete suite.
- Admission changes the inventory and all lock consumers atomically; the renewed F03.1 source gate
  may pass while all three profiles remain visibly `matrix-incomplete`.

The current inventory permits one entry for each exact source family. F02.4.4 must either add the
six reviewed logical families coherently or make a separately tested schema migration; it must not
silently reuse an unrelated family or weaken cardinality validation.
