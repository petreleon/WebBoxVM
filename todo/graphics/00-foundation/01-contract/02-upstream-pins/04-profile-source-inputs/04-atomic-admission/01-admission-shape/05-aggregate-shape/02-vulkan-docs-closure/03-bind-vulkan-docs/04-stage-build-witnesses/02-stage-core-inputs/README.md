# F02.4.4.1.5.2.3.4.2 — Stage and verify captured core inputs

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.4.2
Depends: F02.4.4.1.5.2.3.4.1
Evidence: pending

Prerequisite lists: the [staging contract](../01-stage-contract/README.md), [bound input scope](../../03-capture-core-closure/02-bind-core-input-scope/README.md),
and [independent comparison](../../03-capture-core-closure/03-compare-fresh-captures/README.md).

## Starting points

- [staging-contract task](../01-stage-contract/README.md)
- [bound-scope receipt](../../03-capture-core-closure/02-bind-core-input-scope/evidence.md)
- [comparison receipt](../../03-capture-core-closure/03-compare-fresh-captures/evidence.md)

## Outcome

Rehash the selected 298 raw and 1,462 derived members from both recorded captures, then stage one canonical semantic
input set under distinct raw and generated-input namespaces without admitting it.

## Checklist

- [ ] Descriptor-read every bounded raw and derived provider from both selected captures before accepting a canonical staged member.
- [ ] Require provider bytes, selector, byte count, and SHA-256 to match the bound scope and comparison identities.
- [ ] Keep raw and generated-input namespaces distinct even where a generated input overlaps a rendered-output path.
- [ ] Rehash every staged member after publication and reject absent, changed, wrong-size, wrong-digest, duplicate-selector, scope-crossed, or capture-substituted data.
- [ ] Add positive and hostile provider, scope, identity, cap, selector, and filesystem regressions with a compact receipt.

## Verification

- A staged input is an unadmitted recorded-capture snapshot, not a replacement for either build observation.
- Output payloads and reusable marker publication remain for later children.
