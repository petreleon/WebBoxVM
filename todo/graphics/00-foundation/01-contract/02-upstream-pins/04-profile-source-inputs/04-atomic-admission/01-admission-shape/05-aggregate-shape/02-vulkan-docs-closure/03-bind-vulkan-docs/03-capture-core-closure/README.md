# F02.4.4.1.5.2.3.3 — Capture the core input and scope closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.3
Depends: F02.4.4.1.5.2.3.2
Evidence: pending

Prerequisite lists: the [actual Docs grammar](../02-actual-closure-identity/README.md),
[Docs include audit](../../../../../../03-vulkan-input-audit/README.md), and [Vulkan boundary](../../../../03-vulkan-boundaries/README.md).

## Outcome

Capture every cap-valid raw/generated core closure input and the exact WSI, video, extension, configuration, and
promotion-metadata treatment from the reproducible official build without treating a root or observations as a closure.

## Starting points

- [build witness](../01-reproduce-pinned-html/evidence.md)
- [direct include transcript](../../../../../../03-vulkan-input-audit/spec_includes.json)
- [boundary requirements](../../../../03-vulkan-boundaries/boundaries.json)

## Checklist

- [ ] Enumerate ordered raw and generated closure inputs with immutable identities and producers.
- [ ] Bind exact core configuration, exclusions, required promotion metadata, images, and selectors.
- [ ] Reject root-only, omitted, WSI/video/extension-expanded, malformed, or over-limit input closures.
- [ ] Compare the captured source/input closure independently across both fresh builds.
- [ ] Record a reproducible unadmitted input/scope receipt with nonzero counts.

## Verification

- A command-line core flag alone is not evidence that every included and excluded member has the required scope.
- No rendered output, active inventory, candidate decision, F03 state, or support claim changes in this child.
