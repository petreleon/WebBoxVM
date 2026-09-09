# F02.4.4.1.5.3 — Resolve the VCTS Vulkan-core closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.4.4.1.5.3
Depends: F02.2, F02.4.3, F02.4.4.1.3
Evidence: [blocker record](evidence.md)

Prerequisite lists: the [VCTS boundary](../../03-vulkan-boundaries/README.md),
[F02 source policy](../../../../../02-fetch-verifier/01-fetch-contract/source_model.py), and
[VCTS audit](../../../../03-vulkan-input-audit/README.md).

## Outcome

Either establish a complete F02.2-valid Vulkan 1.4 core mustpass closure or retain the exact blocker.
The broad `vk-default` root is never an eligible substitute for recursively pinned, core-only members.

## Starting points

- [direct reference transcript](../../../../03-vulkan-input-audit/mustpass_references.json)
- [boundary requirements](../../03-vulkan-boundaries/boundaries.json)
- [source fetch policy](../../../../../02-fetch-verifier/01-fetch-contract/source_fetch.py)

## Checklist

- [ ] Enumerate all selected recursive core members and their selectors without importing WSI, video, or extension groups.
- [ ] Bind every member to F02.2 URL, revision, digest, bytes, license, cache, and selector policy.
- [ ] Resolve the 14 over-limit observations under F02.2's 8 MiB limit without weakening the policy or splitting opaque source roots.
- [ ] Reject root-only, unpinned, oversize, incomplete, or scope-expanded data with focused tests.
- [ ] Record either a reproducible complete closure proof or the first concrete unresolved blocker.

## Verification

- The existing 98 observations and 434,669,348 aggregate inspected bytes are not an immutable closure.
- No inventory, candidate decision, F03 state, guest API, browser, CTS, conformance, or performance claim changes here.
- Status: **BLOCKED** by the absence of an upstream Vulkan-1.4-core selector and policy-violating
  `vk-default` members; the blocker record preserves the reproduction and does not check this task off.
