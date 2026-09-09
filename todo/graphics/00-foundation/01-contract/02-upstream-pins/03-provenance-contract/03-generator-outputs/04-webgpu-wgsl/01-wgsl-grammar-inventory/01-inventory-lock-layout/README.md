# F02.3.3.4.1.1 — Migrate the inventory to a composite lock

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.3.3.4.1.1
Depends: F02.1, F02.2
Evidence: pending

Prerequisite lists: [F02.1](../../../../../01-input-inventory/README.md) and
[F02.2](../../../../../02-fetch-verifier/README.md).

## Outcome

The existing 15 immutable input entries move into readable bounded files, while one checked-in
canonical lock binds every raw component byte and becomes the sole F02 provenance revision source.

## Starting points

- [current single-file inventory](../../../../../01-input-inventory/manifest.toml)
- [F02.2 manifest loader](../../../../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [F02.3 record validator](../../../../01-provenance-record/provenance_record.py)
- [F06 reproducibility proof](../../../../../../../02-reproducibility/03-file-layout/03-generation-and-checker/03-reproducibility-proof/README.md)

## Checklist

- [ ] Define schema-versioned root metadata, bounded entry files, and a deterministic checked-in lock.
- [ ] Make the lock bind the root and every entry file by name and exact raw SHA-256 without source
  payloads, absolute paths, timestamps, or downstream generated output.
- [ ] Make F02.1, F02.2, F02.3, and F06 consume that one lock identity fail-closed.
- [ ] Migrate the current 15 entries unchanged and prove regenerated lock bytes and all focused suites
  are deterministic.

## Verification

- Reordering, omitting, renaming, or changing a component byte invalidates the canonical lock.
- The lock migration alone does not add a new source, change an existing input identity, or claim a
  graphics feature.
