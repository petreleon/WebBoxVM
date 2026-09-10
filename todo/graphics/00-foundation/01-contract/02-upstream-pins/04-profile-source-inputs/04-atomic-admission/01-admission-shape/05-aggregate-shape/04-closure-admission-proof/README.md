# F02.4.4.1.5.4 — Prove the aggregate closure admission

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4
Depends: F02.4.4.1.5.1, F02.4.4.1.5.5
Evidence: pending

Prerequisite lists: every [aggregate child](../README.md), the
[transition grammar](../../04-post-cutover-rules/post_cutover_schema.py), and [F02.2](../../../../../02-fetch-verifier/README.md).

## Outcome

One final fail-closed aggregate may derive an admission-eligible closure state only after all six
logical inputs are complete, immutable, scoped, and valid under their V1 or V2 source policy. It
cannot claim the later inventory cutover, cache proof, or F03 transition.

## Starting points

- [pre-admission aggregate](../01-pre-admission-aggregate/README.md)
- [V2 source contract](../05-vulkan-source-contract-v2/README.md)

## Checklist

- [ ] Reconcile exactly one complete closure identity for every canonical profile, role, and required input.
- [ ] Validate typed scope/exclusion joins, predecessor shape digest, audit-history adapter, and each
  applicable V1/V2 policy without treating a superseded record as PASS.
- [ ] Reject missing, duplicate, root-only, stale, mixed-lock, or partial closure/receipt inputs.
- [ ] Derive no inventory-ready or F03-ready result; attach focused positive and hostile evidence.

## Verification

- This child may pass only after the V2 contract has concrete PASS evidence and proves its own inputs.
- A retained Docs provenance boundary, stale V2 receipt, or incomplete VCTS closure keeps this list,
  its parent, and F02.4.4.2 incomplete.
