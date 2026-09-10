# F02.4.4.1.5.4.4.1 — Define the derived-Docs successor policy

[Parent task](../README.md)

Task: F02.4.4.1.5.4.4.1
Depends: F02.4.4.1.5.4.3, F02.4.4.1.5.5
Evidence: pending

Prerequisite lists: the [blocked-state receipt](../../03-blocked-state-receipt/README.md), the
[V2 source contract](../../../05-vulkan-source-contract-v2/README.md), and the existing
[Docs closure evidence](../../../02-vulkan-docs-closure/README.md).

## Outcome

One versioned, policy-only envelope defines how a pinned Vulkan-Docs tree may later supply the
`vulkan-14-spec` source role without pretending that a generated document is a raw F02.2 input or
that VCTS is a core-only selector. The target remains Vulkan 1.4 core.

## Starting points

- [Vulkan Docs actual-closure grammar](../../../02-vulkan-docs-closure/03-bind-vulkan-docs/02-actual-closure-identity/README.md)
- [V2 canonical-suite boundary](../../../05-vulkan-source-contract-v2/01-policy-decision/README.md)
- [F03 source requirements](../../../../../../../../03-feature-matrix/01-profile-scope/source_requirements.json)

## Checklist

- [ ] Freeze the exact raw-tree, recipe, builder, configuration, two-run, and core-scope identities.
- [ ] Require every raw source member to meet the 8 MiB cap while keeping rendered artifacts separately bounded.
- [ ] Define authoritative per-member license, attribution, role, provenance, and producer-lineage obligations.
- [ ] Forbid root-only, mutable, alias, output-as-source, VCTS-as-Docs, and false-ready records.
- [ ] Keep inventory, cache freshness, F03, support, conformance, certification, and performance state false.
- [ ] Add focused positive and hostile policy tests with a bounded decision receipt.

## Verification

- A policy record is not a Docs closure and cannot set `admission_eligible`, `admitted`, or
  `cutover_ready`.
- It must preserve V2 `vk-default` as a broader-than-core canonical diagnostic suite and leave every
  GLES/VCTS blocker visible.
