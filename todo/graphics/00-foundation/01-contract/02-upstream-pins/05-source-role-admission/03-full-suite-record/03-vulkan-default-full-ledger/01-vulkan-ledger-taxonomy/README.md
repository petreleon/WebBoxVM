# F02.5.3.3.1 — Bind the Vulkan ledger and observed taxonomy

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.5.3.3.1
Depends: F02.5.3.1
Evidence: pending

## Outcome

The 98 ordered members of immutable `vk-default.txt` are bound to their Git blob,
byte, and SHA-256 identities. Observed WSI, video, and extension paths are retained;
every unclassified path stays unknown rather than becoming inferred Vulkan core.

## Starting points

- [canonical root](../../01-canonical-full-suite-roots/full_suite_roots.py)
- [captured VCTS ledger](../../../../04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/03-external-closure-cache/02-live-closure/vcts_closure_ledger.json)

## Checklist

- [ ] Bind all 98 ordered paths to the released root and one revision.
- [ ] Record each blob SHA-1, bytes, and SHA-256 plus the aggregate identity.
- [ ] Preserve observed WSI, video, and extension taxonomy with unknown remainder.
- [ ] Reject missing, duplicate, reordered, mixed-revision, or altered member facts.
- [ ] Make no CTS, core-only, or qualification claim.

## Verification

`make graphics-vulkan-ledger-taxonomy-test` must validate exact count, order, aggregate,
taxonomy, and hostile mutations without fetching or executing CTS.
