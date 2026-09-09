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

- [x] [F02.3.3.4.1.1.1 — Define the shared composite inventory layout](01-shared-layout-loader/README.md)
- [ ] [F02.3.3.4.1.1.2 — Adopt the versioned inventory in F02.1 and F02.2](02-f02-consumer-adoption/README.md)
- [ ] [F02.3.3.4.1.1.3 — Prepare provenance consumers for the inventory revision](03-provenance-consumer-transition/README.md)
- [ ] [F02.3.3.4.1.1.4 — Prepare reproducibility consumers for the inventory revision](04-f06-reproducibility-transition/README.md)
- [ ] [F02.3.3.4.1.1.5 — Cut over components, records, and reproducibility evidence atomically](05-atomic-lock-cutover/README.md)

## Verification

- Reordering, omitting, renaming, or changing a component byte invalidates the canonical lock.
- The lock migration alone does not add a new source, change an existing input identity, or claim a
  graphics feature.

## Split rationale

The lock format needs independent hostile-fixture tests before it can replace the active inventory.
The three consumer preparations can stay compatible with the current inventory and have disjoint
owners. The final cutover then updates F02.1, F02.2, F02.3, and F06 together: moving only the
manifest would make existing sidecars or reproducibility checks describe a different identity. The
grammar input remains in the next sibling task, so this split neither fetches a source nor changes
the 15 accepted input identities.
