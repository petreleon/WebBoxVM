# F02.4.4.1.5.4.4.2.2 — Establish derived-member authority

[Parent task](../README.md)

Task: F02.4.4.1.5.4.4.2.2
Depends: F02.4.4.1.5.4.4.2.1
Evidence: [blocker record](evidence.md)

Prerequisite lists: the [historical anchor](../01-successor-closure-anchor/README.md) and the retained
[metadata blocker](../../../../02-vulkan-docs-closure/03-bind-vulkan-docs/05-actual-closure-proof/01-authoritative-member-metadata/README.md).

## Outcome

One authoritative, applicability-bound manifest must provide license expression, attribution, role,
provenance, and producer authority for every derived Docs member. A local default or a raw-source label
cannot fill a derived field.

## Starting points

- [current metadata blocker](../../../../02-vulkan-docs-closure/03-bind-vulkan-docs/05-actual-closure-proof/01-authoritative-member-metadata/evidence.md)
- [source policy](../../01-derived-docs-policy/README.md)

## Checklist

- [ ] Identify an authoritative upstream source and scope for every derived member field.
- [ ] Bind each entry to exact derived selector, digest, bytes, and producer generation identity.
- [ ] Require complete license, attribution, role, provenance, and producer coverage without defaults.
- [ ] Reject raw-to-derived inheritance, partial manifests, stale sources, aliases, and invented labels.
- [ ] Preserve all source, inventory, F03, support, conformance, certification, and performance effects false.
- [x] Retain the first external-authority blocker with hostile proof.

## Verification

- Current evidence is blocked: 1,462 derived members lack authoritative license, role, provenance, and
  producer evidence. This child stays unchecked until that authority exists and applies to each member.
