# F02.4.4.1.4 — Define post-cutover rules

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.4.4.1.4
Depends: F02.1, F02.2, F02.4.4.1.2, F02.4.4.1.3
Evidence: pending

Prerequisite lists: the required-source map, Vulkan boundaries, and
[inventory layout](../../../../01-input-inventory/inventory_layout.py).

## Outcome

A separate fail-closed transition design distinguishes pre-admission audit decisions from a future
atomic inventory state. It neither weakens the candidate audit nor permits a partial inventory change.

## Starting points

- [pre-admission candidate contract](../../../candidate_contract.py)
- [inventory layout](../../../../01-input-inventory/inventory_layout.py)
- [F03 source gate](../../../../../03-feature-matrix/01-profile-scope/profile_contract.py)

## Checklist

- [ ] Define the only valid pre-admission and post-cutover states for each source shape.
- [ ] Require a complete closure identity before a compound logical ID can cross the boundary.
- [ ] Keep candidate audit rejection valid before cutover without mutating its schema or meaning.
- [ ] Reject mixed pre/post records, partial source sets, stale locks, and direct root substitution.
- [ ] Specify the atomic consumers that F02.4.4.2–F02.4.4.5 must renew together.

## Verification

- No rule provides an accepted state through a mutable constructor, old candidate lock, or root alone.
- This is a source-transition design, not profile support, a guest interface, or a graphics result.
