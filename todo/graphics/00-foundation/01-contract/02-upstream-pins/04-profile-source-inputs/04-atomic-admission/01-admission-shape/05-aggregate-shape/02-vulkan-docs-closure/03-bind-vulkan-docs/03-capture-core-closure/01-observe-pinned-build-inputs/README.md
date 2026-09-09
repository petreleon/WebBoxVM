# F02.4.4.1.5.2.3.3.1 — Observe pinned core build inputs

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.3.1
Depends: F02.4.4.1.5.2.3.2
Evidence: pending

Prerequisite lists: the [actual Docs grammar](../../02-actual-closure-identity/README.md),
[official build witness](../../01-reproduce-pinned-html/evidence.md), and
[Vulkan boundary](../../../../../03-vulkan-boundaries/boundaries.json).

## Outcome

An explicit, replayable observer records the content inputs and conditional include resolution for the pinned core
build. It distinguishes generator, Asciidoctor, and copied/postprocess asset phases; it is evidence beside, not a
replacement for, the official output witness.

## Starting points

- [pinned build recipe](../../02-actual-closure-identity/vulkan_docs_identity_build.py)
- [direct include transcript](../../../../../../../03-vulkan-input-audit/spec_includes.json)
- [source boundary record](../../../../../03-vulkan-boundaries/boundaries.json)

## Checklist

- [ ] Record the exact core producer argv and effective attributes before observing any inputs.
- [ ] Observe generator reads, resolved Asciidoctor includes, and copied/postprocess assets with pinned image, read-only
  inputs, no network, and a declared observer implementation.
- [ ] Normalize only content reads into ordered phase records with safe source/generated selectors and no rendered output
  classified as an input.
- [ ] Retain two ignored raw observation artifacts plus compact tracked identities; reject incomplete or unsafe observer
  records in focused tests.

## Verification

- An include hook alone cannot stand in for generator or KaTeX asset observations.
- The observer does not alter the official recipe, claim a rendered tree, or materialize a successor cache.
