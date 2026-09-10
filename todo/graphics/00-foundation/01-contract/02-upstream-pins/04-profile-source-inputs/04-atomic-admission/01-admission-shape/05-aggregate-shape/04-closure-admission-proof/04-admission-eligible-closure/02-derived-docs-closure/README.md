# F02.4.4.1.5.4.4.2 — Prove the derived-Docs source closure

[Parent task](../README.md)

Task: F02.4.4.1.5.4.4.2
Depends: F02.4.4.1.5.4.4.1
Evidence: pending

Prerequisite lists: the [derived-Docs policy](../01-derived-docs-policy/README.md) and the retained
[Docs closure blocker](../../../02-vulkan-docs-closure/03-bind-vulkan-docs/05-actual-closure-proof/README.md).

## Outcome

Two fresh, pinned, isolated core builds establish one complete, immutable Vulkan-Docs source closure
under the successor policy. The proof keeps generated artifacts distinct from raw sources and remains
unadmitted until all aggregate roles are independently valid.

## Starting points

- [core-input capture](../../../02-vulkan-docs-closure/03-bind-vulkan-docs/03-capture-core-closure/README.md)
- [staged build witnesses](../../../02-vulkan-docs-closure/03-bind-vulkan-docs/04-stage-build-witnesses/README.md)
- [metadata blocker](../../../02-vulkan-docs-closure/03-bind-vulkan-docs/05-actual-closure-proof/01-authoritative-member-metadata/README.md)

## Checklist

- [ ] Capture matching raw/derived source, recipe, builder, configuration, scope, and rendered-output identities.
- [ ] Prove complete authoritative license, attribution, role, provenance, and producer lineage for every member.
- [ ] Rehash all cap-valid inputs in an isolated external cache and reject mutations, omissions, aliases, and races.
- [ ] Preserve exact Vulkan 1.4 core/WSI/video/extension treatment without scope expansion or erosion.
- [ ] Reject root-only, provenance-only, output-as-input, stale-run, and partial-closure evidence.
- [ ] Attach focused positive and hostile PASS evidence without setting aggregate admission or F03 state.

## Verification

- A complete Docs closure proves only the `vulkan-14-spec` source role under its successor envelope.
- The independent GLES CTS and Vulkan CTS obligations remain live; this child cannot call the product
  compatible, conformant, certified, or near-native.
