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
first freezes its grammar, then seals the actual blocked state; neither a passing validator nor a
recorded VCTS cache can imply inventory cutover, fresh cache proof, or F03 transition.

## Starting points

- [pre-admission aggregate](../01-pre-admission-aggregate/README.md)
- [V2 source contract](../05-vulkan-source-contract-v2/README.md)

## Checklist

- [ ] [F02.4.4.1.5.4.1 — Freeze the admission-proof contract](01-proof-contract/README.md)
- [ ] [F02.4.4.1.5.4.2 — Reconcile current evidence fail-closed](02-reconcile-current-evidence/README.md)
- [ ] [F02.4.4.1.5.4.3 — Seal the blocked-state receipt](03-blocked-state-receipt/README.md)
- [ ] [F02.4.4.1.5.4.4 — Establish an admission-eligible closure](04-admission-eligible-closure/README.md)

## Verification

- A contract, reconciler, or blocked-state receipt may PASS for faithfully preserving a bounded state;
  that is not an admission PASS. The final child may pass only after every live role has a valid policy.
- A retained Docs provenance boundary, stale V2 receipt, or incomplete VCTS closure keeps this list,
  its parent, and F02.4.4.2 incomplete.
