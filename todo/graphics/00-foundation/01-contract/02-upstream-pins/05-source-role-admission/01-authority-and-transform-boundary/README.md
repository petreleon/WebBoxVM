# F02.5.1 — Define source authority and transform roles

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.5.1
Depends: F02.1, F02.2, F02.3.1, F03.1
Evidence: pending

## Outcome

One fail-closed schema distinguishes an immutable upstream source, a WebBoxVM-produced transform,
and an unmodified full-suite root. It defines which claims each role may make and preserves the
8 MiB limit for every WebBoxVM-maintained source fragment or transport shard.

## Starting points

- [F02 fetch model](../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [Vulkan audit](../../04-profile-source-inputs/03-vulkan-input-audit/README.md)
- [source requirements](../../../03-feature-matrix/01-profile-scope/source_requirements.json)

## Checklist

- [ ] Specify required immutable identity, digest, license, attribution, role, producer, and scope fields.
- [ ] Require local transforms to name WebBoxVM as producer and their exact input identities and command.
- [ ] Forbid a local core map, shard, or derived document from claiming Khronos authorship or conformance.
- [ ] Define full-suite roots as unfiltered upstream selectors plus their pinned source revision.
- [ ] Add positive and hostile schema tests, including an over-8-MiB local fragment and a false authority claim.
- [ ] Run focused tests and attach a receipt with exact source and policy boundaries.

## Verification

- Invalid role transitions, mutable identities, missing attribution, oversized local fragments, and
  authority/conformance overclaims fail before any inventory or matrix consumer accepts the record.
