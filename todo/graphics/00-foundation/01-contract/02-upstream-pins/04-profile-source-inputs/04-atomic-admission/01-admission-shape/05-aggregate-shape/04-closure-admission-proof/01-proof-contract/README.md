# F02.4.4.1.5.4.1 — Freeze the admission-proof contract

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.1
Depends: F02.4.4.1.5.1, F02.4.4.1.5.5
Evidence: pending

Prerequisite lists: the [pre-admission aggregate](../../01-pre-admission-aggregate/README.md),
[V2 handoff](../../05-vulkan-source-contract-v2/05-aggregate-handoff/README.md), and
[F03 source requirements](../../../../../../../03-feature-matrix/01-profile-scope/README.md).

## Outcome

One immutable grammar fixes the exact six ordered `(profile, role, required_input_id)` records and
their tagged V1/V2 evidence envelopes. It separates valid evidence from admission, cutover,
fresh-cache, and F03 readiness, all of which remain false for the current state.

## Starting points

- [pre-admission contract](../../01-pre-admission-aggregate/aggregate_contract.py)
- [V2 handoff validator](../../05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff.py)
- [post-cutover contract](../../../04-post-cutover-rules/post_cutover_contract.py)

## Checklist

- [ ] Fix the canonical six profile, role, and required-ID tuples in their F03 order with no aliases or surplus.
- [ ] Define distinct regular-source and canonical-suite envelopes without inventing per-member V2 F02.2 fields.
- [ ] Prohibit V2 from satisfying `vulkan-14-spec` or recasting `vk-default` as a Vulkan-1.4-core selector.
- [ ] Enforce terminal false readiness flags and reject missing, reordered, duplicate, mixed-policy, or extra fields.
- [ ] Run focused positive and hostile grammar tests and attach a small receipt.

## Verification

- A PASS here proves a fail-closed grammar, not a source admission. It must accept a correctly blocked
  state and reject a fixture that makes V2 or a Docs observation appear admitted.
- No inventory, source map, cache, F03, guest API, browser, conformance, or performance output changes here.
