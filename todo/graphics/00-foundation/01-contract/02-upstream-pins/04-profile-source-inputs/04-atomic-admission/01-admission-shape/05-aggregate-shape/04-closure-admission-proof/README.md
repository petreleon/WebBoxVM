# F02.4.4.1.5.4 — Prove the aggregate closure admission

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4
Depends: F02.4.4.1.5.1, F02.4.4.1.5.2, F02.4.4.1.5.3
Evidence: pending

Prerequisite lists: every [aggregate child](../README.md), the
[transition grammar](../../04-post-cutover-rules/post_cutover_schema.py), and [F02.2](../../../../../02-fetch-verifier/README.md).

## Outcome

One final fail-closed aggregate may derive an admission-eligible closure state only after all six
logical inputs are complete, immutable, scoped, and policy-valid. It cannot claim the later inventory
cutover, cache proof, or F03 transition.

## Starting points

- [pre-admission aggregate](../01-pre-admission-aggregate/README.md)
- [Docs closure](../02-vulkan-docs-closure/README.md)
- [VCTS closure](../03-vcts-core-closure/README.md)

## Checklist

- [ ] Reconcile exactly one complete closure identity for every canonical profile, role, and required input.
- [ ] Validate typed scope/exclusion joins, predecessor shape digest, audit history adapter, and F02.2 member policy.
- [ ] Reject missing, duplicate, root-only, stale, mixed-lock, or partial closure/receipt inputs.
- [ ] Derive no inventory-ready or F03-ready result; attach focused positive and hostile evidence.

## Verification

- This child may pass only after both Vulkan closure children have concrete PASS evidence.
- Any remaining Docs or VCTS blocker keeps this list, its parent, and F02.4.4.2 incomplete.
