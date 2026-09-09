# F06.3.3 — Verify generated metadata and reproducibility

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F06.3.3
Depends: F06.3.1, F06.3.2
Evidence: pending

Prerequisite lists: [F06.3.1](../01-chunk-generator/README.md) and
[F06.3.2](../02-checker-fixtures/README.md).

## Outcome

One reproducible evidence run proves generated output is stable, metadata is current, and the
expanded roadmap checker remains structurally deterministic.

## Starting points

- [chunk generator](../01-chunk-generator/README.md)
- [checker fixtures](../02-checker-fixtures/README.md)
- [F02 provenance contract](../../../../01-contract/02-upstream-pins/03-provenance-contract/README.md)

## Checklist

- [ ] Generate the same ordered input twice and record matching chunk/metadata SHA-256 values.
- [ ] Prove stale metadata and a changed ordering fail the checker command.
- [ ] Run generator, fixture suite, source limit, roadmap, and whitespace checks.
- [ ] Record fixtures, exact output hashes, revisions, and blockers in a receipt.

## Verification

- Repeated deterministic generation is byte-identical and every generated chunk is within 180 lines.
- A stale or reordered input is rejected, while the real roadmap and fixture suite both pass.
