# F02.4.4.1.5.2 — Resolve the Vulkan Docs core closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2
Depends: F02.2, F02.4.3, F02.4.4.1.3
Evidence: pending

Prerequisite lists: the [Vulkan boundary](../../03-vulkan-boundaries/README.md),
[F02 source policy](../../../../../02-fetch-verifier/01-fetch-contract/source_model.py), and
[Vulkan Docs audit](../../../../03-vulkan-input-audit/README.md).

## Outcome

Either establish a complete, F02.2-valid Vulkan 1.4 core document closure or retain a precise blocker.
Generated/transitive files, macro/conditional configuration, and core-only scope must all have bounded
immutable identities; the root alone never qualifies.

## Starting points

- [direct include transcript](../../../../03-vulkan-input-audit/spec_includes.json)
- [boundary requirements](../../03-vulkan-boundaries/boundaries.json)
- [source fetch policy](../../../../../02-fetch-verifier/01-fetch-contract/source_fetch.py)

## Checklist

- [ ] Enumerate the recursive generated/transitive core closure from the pinned Docs root without treating observations as members.
- [ ] Bind every candidate member to F02.2 URL, revision, digest, bytes, license, cache, and selector policy.
- [ ] Bind macro/conditional configuration and preserve WSI, video, and extension exclusions exactly.
- [ ] Reject root-only, incomplete, mutable, oversize, or scope-expanded closure data with focused tests.
- [ ] Record either a reproducible complete closure proof or the first concrete unresolved blocker.

## Verification

- The existing 73 observations are insufficient; this task stays open unless every required Docs member and scope input is genuinely modelled.
- No inventory, candidate decision, F03 state, guest API, browser, CTS, conformance, or performance claim changes here.
