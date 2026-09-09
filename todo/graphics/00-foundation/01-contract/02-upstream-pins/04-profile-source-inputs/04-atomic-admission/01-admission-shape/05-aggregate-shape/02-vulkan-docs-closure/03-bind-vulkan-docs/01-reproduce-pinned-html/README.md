# F02.4.4.1.5.2.3.1 — Reproduce pinned official core HTML

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.1
Depends: F02.4.4.1.5.2.2
Evidence: [receipt](evidence.md)

Prerequisite lists: the [successor cache child](../../02-successor-cache-verifier/README.md) and
[Docs progress record](../evidence.md).

## Outcome

Reproduce the pinned official Vulkan-Docs Vulkan 1.4 core HTML route twice in fresh, network-isolated environments
and compare their complete generated trees. This is a build witness only, never source admission.

## Starting points

- [pinned Docs discovery](../../evidence.md)
- [official build progress](../evidence.md)
- [Vulkan boundary](../../../../03-vulkan-boundaries/README.md)

## Checklist

- [x] Use two clean detached upstream checkouts and the exact pinned official image.
- [x] Bind route, platform, internal paths, locale, environment, and dynamic build attributes.
- [x] Compare every generated file from both clean runs and retain only ignored reproducible artifacts.
- [x] Record commands, identities, nonzero output counts, and the exact output-tree result.

## Verification

- A login-shell `PATH` change must not be misreported as an absent pinned-image dependency.
- The witness has no source inventory, cache-admission, core-scope-completeness, guest, browser, CTS, or performance claim.
