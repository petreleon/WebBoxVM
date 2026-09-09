# Q06 — Finish with traceable commits and an accurate handoff

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: Q06
Depends: Q05
Evidence: pending

Prerequisite lists: [Q05](../02-publish-evidence/README.md).

## Outcome

All completed work is reviewable remotely and unfinished work remains explicit.

## Starting points

- [todo.md](../../../../../todo.md)
- [sprint-history.md](../../../../../sprint-history.md)
- [Makefile](../../../../../Makefile)

## Checklist

- [ ] Check every completed leaf has a receipt, valid dependencies and correct parent rollups.
- [ ] Commit only task-owned files after their required checks; push normal focused commits and
  verify the remote revision.
- [ ] Record final branch/SHA, exact passing profiles and benchmark results; list any blockers
  without marking them complete.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run git diff --check and python3 scripts/check_graphics_roadmap.py; compare the pushed commit with
  git ls-remote.
- Overall completion requires all mandatory compatibility and near-native performance gates, not
  merely a finished roadmap.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
