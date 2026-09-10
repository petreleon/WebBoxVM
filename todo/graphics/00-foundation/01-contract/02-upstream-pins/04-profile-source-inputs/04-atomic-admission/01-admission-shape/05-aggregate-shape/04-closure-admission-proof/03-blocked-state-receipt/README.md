# F02.4.4.1.5.4.3 — Seal the blocked-state receipt

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.3
Depends: F02.4.4.1.5.4.2
Evidence: pending

Prerequisite lists: the [current evidence reconciler](../02-reconcile-current-evidence/README.md)
and [parent admission proof](../README.md).

## Outcome

One small self-hashed diagnostic receipt binds the actual V1 aggregate and V2 handoff validation
into a reproducible proof of a correctly blocked admission state. It is immutable evidence of what
was checked, not a future successor inventory, cache proof, cutover, or F03 decision.

## Starting points

- [current evidence reconciler](../02-reconcile-current-evidence/README.md)
- [V2 handoff receipt](../../05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff.json)
- [receipt template](../../../../../../../../../evidence-template.md)

## Checklist

- [ ] Seal the typed blocked result and every referenced input digest under one bounded, self-hashed receipt.
- [ ] Require no inventory-ready, fresh-cache-ready, cutover-ready, or F03-ready flags in the receipt.
- [ ] Reject tampered receipt fields, stale inputs, output self-hash mismatch, and any ready-state substitution.
- [ ] Demonstrate that validation and receipt sealing do not change source-map, inventory, or F03 artifacts.
- [ ] Run focused CLI and hostile tests and attach a small receipt.

## Verification

- A PASS means only that this correctly blocked state is sealed and reproducible. Its profile must say
  `proof-of-correctly-blocked-admission-state`, never successful source admission.
- The parent remains open: the retained Docs provenance boundary is a material blocker, not an expected skip.
