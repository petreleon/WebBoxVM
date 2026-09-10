# F02.4.4.1.5.2.3.5.2.5 — Bind the collector to an authorized Docs replay

[Parent task](../README.md)

Task: F02.4.4.1.5.2.3.5.2.5
Depends: F02.4.4.1.5.2.3.5.1, F02.4.4.1.5.2.3.5.2.4
Evidence: pending

## Outcome

Only after authoritative per-member Docs metadata exists, bind the collector to two fresh isolated official replay
runs without relaxing the source/image/network policy or treating a synthetic trace as a Docs trace.

## Starting points

- [metadata blocker](../../01-authoritative-member-metadata/README.md)
- [synthetic hosted witness](../04-hosted-synthetic-witness/README.md)
- [fresh-capture task](../../03-capture-fresh-lineage/README.md)

## Checklist

- [ ] Require the authorized metadata manifest before selecting any source or derived Docs member.
- [ ] Anchor source tree, collector, replay recipe, outputs, and trace artifacts before accepting either run.
- [ ] Produce two fresh normalized traces that bind every actual derived member to one writer and producers.
- [ ] Preserve explicit daemon, branch, and attestation limitations until an authorized trust route resolves them.

## Verification

- A local metadata rule, old capture, fixture trace, or unanchored Docker path mount cannot satisfy this child.
- The sibling fresh-lineage task remains the sole place to claim an actual Docs capture.
