# F02.4.4.1.5.2.3 — Bind the actual Vulkan Docs closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3
Depends: F02.4.4.1.5.2.2
Evidence: [progress record](evidence.md)

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

- [x] [F02.4.4.1.5.2.3.1 — Reproduce pinned official core HTML](01-reproduce-pinned-html/README.md)
- [x] [F02.4.4.1.5.2.3.2 — Define the actual Docs closure identity](02-actual-closure-identity/README.md)
- [x] [F02.4.4.1.5.2.3.3 — Capture the core input and scope closure](03-capture-core-closure/README.md)
- [ ] [F02.4.4.1.5.2.3.4 — Stage and verify build witnesses](04-stage-build-witnesses/README.md)
- [ ] [F02.4.4.1.5.2.3.5 — Prove the actual Docs closure](05-actual-closure-proof/README.md)

## Verification

- The current generator-only discovery is not sufficient: this child stays open until official fresh builds agree.
- No successor inventory, candidate decision, F03 state, guest API, browser, CTS, conformance, or performance claim changes here.
