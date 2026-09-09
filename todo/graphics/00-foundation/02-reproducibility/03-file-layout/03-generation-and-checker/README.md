# F06.3 — Make generated output and roadmap nesting deterministic

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F06.3
Depends: F06.2
Evidence: pending

Prerequisite lists: [F06.2](../02-line-limit-coverage/README.md).

## Outcome

Generated protocol output is chunked reproducibly and deeper roadmap lists remain checked structurally.

## Starting points

- [line-limit coverage](../02-line-limit-coverage/README.md)
- [roadmap checker](../../../../../../scripts/check_graphics_roadmap.py)
- [F02 provenance contract](../../../01-contract/02-upstream-pins/03-provenance-contract/README.md)

## Checklist

- [x] [F06.3.1 — Build deterministic generated-file chunking](01-chunk-generator/README.md)
- [x] [F06.3.2 — Add isolated roadmap-checker regressions](02-checker-fixtures/README.md)
- [ ] [F06.3.3 — Verify generated metadata and reproducibility](03-reproducibility-proof/README.md)

## Verification

- The same inputs generate byte-identical ordered chunks; a 181-line generated chunk is rejected.
- Valid nested lists pass while each malformed fixture fails with the expected first diagnostic.
