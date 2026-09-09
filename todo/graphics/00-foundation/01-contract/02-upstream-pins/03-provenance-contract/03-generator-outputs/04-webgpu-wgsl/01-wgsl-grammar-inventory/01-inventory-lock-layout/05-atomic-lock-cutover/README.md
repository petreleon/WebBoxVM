# F02.3.3.4.1.1.5 — Cut over components, records, and reproducibility evidence atomically

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.3.3.4.1.1.5
Depends: F02.3.3.4.1.1.2, F02.3.3.4.1.1.3, F02.3.3.4.1.1.4
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.1/F02.2 adoption](../02-f02-consumer-adoption/README.md),
[provenance transition](../03-provenance-consumer-transition/README.md), and
[F06 transition](../04-f06-reproducibility-transition/README.md).

## Outcome

The existing 15 inputs move unchanged to bounded components with one canonical lock. All checked-in
F02.3 sidecars and F06 fixture metadata switch to the new inventory identity in the same verified
commit, leaving no active consumer half-migrated.

## Starting points

- [shared layout contract](../01-shared-layout-loader/README.md)
- [input inventory](../../../../../../01-input-inventory/manifest.toml)
- [F06 fixture](../../../../../../../../02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture/input.json)

## Checklist

- [x] Move the existing 15 inputs unchanged into sorted bounded components, replace the root with
  schema-v2 metadata, and regenerate the checked-in canonical lock.
- [x] Renew every checked-in F02.3 sidecar and F06 fixture/metadata to the one canonical inventory
  revision in the same commit.
- [x] Prove component reorder, omission, rename, byte mutation, stale record, and stale generated
  metadata fail closed.
- [x] Run focused F02.1/F02.2/F02.3/F06 suites, `make test`, source-limit, roadmap, and whitespace
  checks; record component/lock hashes and exact results in this child receipt.

## Verification

- The old one-file manifest revision, a stale sidecar, and stale F06 metadata are each rejected.
- Regenerating the layout and F06 fixture from committed inputs is deterministic.
