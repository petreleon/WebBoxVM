# F02.4.4.1.5.1 — Aggregate the current pre-admission state

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.4.4.1.5.1
Depends: F02.4.4.1.1, F02.4.4.1.2, F02.4.4.1.3, F02.4.4.1.4
Evidence: pending

Prerequisite lists: the [source map](../../02-source-map/README.md),
[Vulkan boundaries](../../03-vulkan-boundaries/README.md), and
[transition rules](../../04-post-cutover-rules/README.md).

## Outcome

One immutable current-state aggregate proves exactly six required IDs and returns only the actual
pre-admission blockers. It does not fabricate a successor inventory, closure receipt, or ready state.

## Starting points

- [source-map contract](../../02-source-map/source_map_contract.py)
- [boundary contract](../../03-vulkan-boundaries/boundary_contract.py)
- [transition contract](../../04-post-cutover-rules/post_cutover_contract.py)

## Checklist

- [ ] Reconcile the canonical six ordered source-shape records through the three prerequisite validators.
- [ ] Preserve three candidate-only direct roots, the four-member/one-exclusion GLES boundary, and 73/98 Vulkan observations.
- [ ] Reject stale/mixed paths, root substitution, partial records, and duplicate JSON before an aggregate result.
- [ ] Return only an immutable `pre-admission`, zero-ready aggregate and focused hostile evidence.
- [ ] Attach the focused receipt with exact command, counts, and limits.

## Verification

- The real positive case is a correctly blocked aggregate, never a fictional `PostCutover` fixture.
- Vulkan observations are not members or closures; all three retained blockers keep F02.4.4.2 unavailable.
