# F02.4.4.1.5.4.2 — Reconcile current evidence fail-closed

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.2
Depends: F02.4.4.1.5.4.1
Evidence: pending

Prerequisite lists: the [proof contract](../01-proof-contract/README.md),
[pre-admission aggregate](../../01-pre-admission-aggregate/README.md), and
[V2 handoff](../../05-vulkan-source-contract-v2/05-aggregate-handoff/README.md).

## Outcome

One read-only reconciler calls both current validators, binds their predecessor identities, and emits
only a typed `blocked` diagnostic for the actual six-role state. It preserves the three V1 blockers,
the V2 suite's broader-than-core scope, and all V2 false state bits.

## Starting points

- [proof contract](../01-proof-contract/README.md)
- [pre-admission contract](../../01-pre-admission-aggregate/aggregate_contract.py)
- [V2 handoff validator](../../05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff.py)

## Checklist

- [ ] Revalidate current V1 and V2 inputs through their own validators, never a copied receipt or self-hash alone.
- [ ] Bind canonical order, predecessor/audit joins, V2 root/tree/ledger/cache/live/taxonomy identities, and false flags.
- [ ] Report exactly the real first blocked role without treating Docs provenance or a V1 record as admission evidence.
- [ ] Reject stale, partial, root-only, scope-eroded, V1/V2-substituted, tampered, or false-ready input.
- [ ] Run focused positive and hostile tests and attach a small receipt.

## Verification

- The valid live result is `blocked`, not `admission_eligible`. VCTS remains a canonical diagnostic
  suite, not Vulkan-1.4-core or conformance evidence, and Docs remains outside implementation closure.
- Do not write an inventory, `admission_closures.json`, `cutover_receipt.json`, cache marker, or F03 state.
