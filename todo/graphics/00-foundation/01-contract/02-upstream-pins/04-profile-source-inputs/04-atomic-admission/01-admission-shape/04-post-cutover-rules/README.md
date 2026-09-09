# F02.4.4.1.4 — Define post-cutover rules

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.4.4.1.4
Depends: F02.1, F02.2, F02.4.4.1.2, F02.4.4.1.3
Evidence: [receipt](evidence.md)

Prerequisite lists: the [required-source map](../02-source-map/README.md),
[Vulkan boundaries](../03-vulkan-boundaries/README.md), and
[inventory layout](../../../../01-input-inventory/inventory_layout.py).

## Outcome

A separate fail-closed transition design distinguishes pre-admission audit decisions from a future
atomic inventory state. It neither weakens the candidate audit nor permits a partial inventory change.
Its future grammar requires an ordered closure for every canonical ID, full F02.2 member identities,
exact compound scope, successor-lock membership, all three audit bindings, and all four renewed
consumer receipts. This child validates that grammar and today's blocked inputs only; F02.4.4.1.5
will validate any future closure/receipt instances before it can expose a cutover-ready result.
The grammar preserves each legacy audit's predecessor lock through a separately pinned successor
adapter; it never pretends that the old candidate audit itself was rebased.

## Starting points

- [pre-admission candidate contract](../../../candidate_contract.py)
- [inventory layout](../../../../01-input-inventory/inventory_layout.py)
- [F03 source gate](../../../../../03-feature-matrix/01-profile-scope/profile_contract.py)
- [transition rules](post_cutover_rules.json)
- [future artifact grammar](post_cutover_schema.py)
- [transition contract](post_cutover_contract.py)
- [transition tests](post_cutover_test.py)

## Checklist

- [x] Define the only valid pre-admission and post-cutover states for each source shape.
- [x] Require a complete closure identity before a compound logical ID can cross the boundary.
- [x] Keep candidate audit rejection valid before cutover without mutating its schema or meaning.
- [x] Reject mixed pre/post records, partial source sets, stale locks, and direct root substitution.
- [x] Specify the atomic consumers that F02.4.4.2–F02.4.4.5 must renew together.

## Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 post_cutover_test.py` must report eight passing tests.
- No rule provides an accepted state through a mutable constructor, old candidate lock, or root alone.
- Future artifact instances are deliberately absent: their hostile closure, successor-inventory, and
  mixed-consumer validation belongs to F02.4.4.1.5, never to an invented current receipt.
- This is a source-transition design, not profile support, a guest interface, or a graphics result.
