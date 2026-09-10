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

- [x] [F02.4.4.1.5.4.4.2.1 — Anchor the historical successor closure evidence](01-successor-closure-anchor/README.md)
- [ ] [F02.4.4.1.5.4.4.2.2 — Establish derived-member authority](02-derived-member-authority/README.md)
- [ ] [F02.4.4.1.5.4.4.2.3 — Capture authorized write lineage](03-authorized-write-lineage/README.md)
- [ ] [F02.4.4.1.5.4.4.2.4 — Rehash and reconcile the successor closure](04-rehash-and-reconcile/README.md)

## Verification

- A complete Docs closure proves only the `vulkan-14-spec` source role under its successor envelope.
- The independent GLES CTS and Vulkan CTS obligations remain live; this child cannot call the product
  compatible, conformant, certified, or near-native.
- Historical witnesses show two matching observations but are not the two fresh authorized replays required
  by the successor policy. Missing per-derived authority and write lineage remain distinct blockers.
