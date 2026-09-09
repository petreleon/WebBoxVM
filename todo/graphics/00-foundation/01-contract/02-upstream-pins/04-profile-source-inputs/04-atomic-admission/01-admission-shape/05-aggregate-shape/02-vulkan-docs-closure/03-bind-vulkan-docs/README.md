# F02.4.4.1.5.2.3 — Bind the actual Vulkan Docs closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3
Depends: F02.4.4.1.5.2.2
Evidence: pending

Prerequisite lists: the [successor cache child](../02-successor-cache-verifier/README.md),
[Docs blocker record](../evidence.md), and [Vulkan boundary](../../../03-vulkan-boundaries/README.md).

## Outcome

Using a fresh environment with the pinned official Docs toolchain, reproduce the full Vulkan 1.4 core document
closure twice and bind every raw/generated member, scope condition, image, and output under the successor contract.

## Starting points

- [pinned Docs discovery](../evidence.md)
- [successor identity child](../01-successor-identity/README.md)
- [successor cache child](../02-successor-cache-verifier/README.md)

## Checklist

- [ ] Run the pinned official `makeSpec -clean -spec core -version 1.4 … html` route twice in fresh environments.
- [ ] Bind source, generated, configuration, toolchain, image, exclusion, and output identities exactly.
- [ ] Prove WSI, video, and extension scope treatment without treating observations or roots as complete members.
- [ ] Reject divergent, missing, root-only, mutable, oversize, partial, or scope-expanded closures.
- [ ] Record a complete proof or the first remaining concrete toolchain/closure blocker.

## Verification

- The current generator-only discovery is not sufficient: this child stays open until official fresh builds agree.
- No successor inventory, candidate decision, F03 state, guest API, browser, CTS, conformance, or performance claim changes here.
