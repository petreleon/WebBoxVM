# F06.3.2 — Add isolated roadmap-checker regressions

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F06.3.2
Depends: F06.2
Evidence: pending

Prerequisite lists: [F06.2](../../02-line-limit-coverage/README.md).

## Outcome

An isolated fixture suite proves nested-roadmap acceptance and each structural diagnostic without
relying on the checked-in roadmap as its only happy path.

## Starting points

- [roadmap checker](../../../../../../../scripts/check_graphics_roadmap.py)
- [graphics workflow](../../../../../workflow.md)
- [line-limit policy](../../02-line-limit-coverage/README.md)

## Checklist

- [ ] Add a valid nested child-list fixture with concrete evidence and dependency metadata.
- [ ] Add malformed depth, broken-link, parent-status, and 181-line fixture cases.
- [ ] Assert deterministic first diagnostics and a nonzero valid/invalid test count.
- [ ] Keep the fixture runner and every fixture source under 180 physical lines.

## Verification

- The valid nested fixture passes through the real checker entrypoint.
- Each malformed fixture fails with its expected first diagnostic, not a hand-inspected result.
