# F02.4.4.1.5.2.3.5.4 — Reify the actual member closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.5.4
Depends: F02.4.4.1.5.2.3.5.1, F02.4.4.1.5.2.3.5.3
Evidence: pending

Prerequisite lists: the [metadata contract](../01-authoritative-member-metadata/README.md),
[fresh lineage](../03-capture-fresh-lineage/README.md), and the [actual Docs grammar](../../02-actual-closure-identity/README.md).

## Outcome

Materialize the first complete, self-hashed `vulkan-docs-actual-closure-v1` manifest from authoritative metadata and
two matching fresh lineage captures, while keeping it strictly unadmitted.

## Starting points

- [closure parser](../../02-actual-closure-identity/vulkan_docs_identity_contract.py)
- [scope rules](../../02-actual-closure-identity/vulkan_docs_identity_scope.py)
- [blocker record](../evidence.md)

## Checklist

- [ ] Build the complete manifest from fact-map and fresh artifacts; retain large payloads outside Git with retrievable hashes.
- [ ] Bind exact input order, scope, witness, configuration, byte identities, and self-hash.
- [ ] Populate each `producer_input_ids` only from captured, topologically earlier lineage evidence.
- [ ] Reject missing, forged, ambiguous, reordered, cyclic, mutated, or scope-expanded closure data in focused tests.
- [ ] Retain `admitted=False` and `cutover_ready=False` in every success and failure receipt.

## Verification

- A fixture-only value or a stage marker cannot substitute for the full actual-grammar manifest.
- The manifest has source-closure meaning only and cannot alter F02 inventory or implementation/support claims.
