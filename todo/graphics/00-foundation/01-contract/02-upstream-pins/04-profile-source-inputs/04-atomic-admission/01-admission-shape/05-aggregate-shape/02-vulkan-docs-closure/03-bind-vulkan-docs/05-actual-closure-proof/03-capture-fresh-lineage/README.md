# F02.4.4.1.5.2.3.5.3 — Capture fresh lineage

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.5.3
Depends: F02.4.4.1.5.2.3.5.1, F02.4.4.1.5.2.3.5.2
Evidence: pending

Prerequisite lists: the [metadata contract](../01-authoritative-member-metadata/README.md),
[lineage collector](../02-proof-lineage-trace/README.md), and [staged witnesses](../../04-stage-build-witnesses/README.md).

## Outcome

Run the exact pinned official Docs build twice in isolated new roots and retain the raw lineage evidence necessary to
prove every observed derived input's writer and conservative producer set.

## Starting points

- [official build witness](../../01-reproduce-pinned-html/evidence.md)
- [current capture route](../../03-capture-core-closure/01-observe-pinned-build-inputs/README.md)
- [staged cache contract](../../04-stage-build-witnesses/README.md)

## Checklist

- [ ] Create two separate fresh source/work roots and retain distinct ignored lineage artifacts with stable retrieval data.
- [ ] Rehash the pinned source, all 1,760 observed inputs, and both full generated/output trees before sealing each run.
- [ ] Require a final writer and conservative producer set for every derived input without reusing an old observer or cache.
- [ ] Compare the two event/lineage identities exactly and record the first differential or unavailable capability.
- [ ] Write a compact unadmitted receipt with commands, revisions, counts, paths, hashes, and negative-case result.

## Verification

- A previous byte-identical capture or receipt is not a fresh lineage capture.
- Same-user artifact provenance and post-final-check TOCTOU limits must remain explicit; no active state may change.
