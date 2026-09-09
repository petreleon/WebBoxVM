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

- [ ] Define deterministic generated-file chunk naming, ordering, headers, and source-manifest
  provenance so reruns have a stable diff and each maintained chunk stays within 180 lines.
- [ ] Add checker regression fixtures for valid nested child lists and invalid depth, links, status,
  or line-count conditions; do not rely on a hand-inspected happy path.
- [ ] Add a generator/checker command that fails on stale generated metadata, unstable ordering, or
  a chunk exceeding the limit.
- [ ] Record the fixture cases, test count, generated-output hashes, and final roadmap-check result.

## Verification

- The same inputs generate byte-identical ordered chunks; a 181-line generated chunk is rejected.
- Valid nested lists pass while each malformed fixture fails with the expected first diagnostic.
