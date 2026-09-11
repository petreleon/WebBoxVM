# F02.5.2 — Record normative source roots and local builds

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.5.2
Depends: F02.5.1
Evidence: pending

## Outcome

The OpenGL 4.6, GLES 3.2, and Vulkan 1.4 normative roots are pinned with their stated terms. Vulkan
core definition is reproduced from pinned Vulkan-Docs and `vk.xml`; the resulting record is explicitly
WebBoxVM-produced rather than an imagined Khronos per-output manifest.

## Starting points

- [candidate catalog](../../04-profile-source-inputs/candidate_catalog.py)
- [Vulkan Docs build instructions](https://github.com/KhronosGroup/Vulkan-Docs/blob/main/BUILD.adoc)
- [Khronos Vulkan registry](https://registry.khronos.org/vulkan/)

## Checklist

- [ ] Re-fetch and verify each immutable normative root, revision, digest, bytes, license, and attribution.
- [ ] Pin the Vulkan core build command, toolchain identity, source inputs, output digest, and source locators.
- [ ] Record `vk.xml` Vulkan-1.4 feature facts separately from normative prose and extension boundaries.
- [ ] Reject mutable, partial, unlicensed, or locally relabeled normative records with focused tests.
- [ ] Attach a receipt that keeps API support, conformance, certification, and performance false.

## Verification

- Rebuilding the declared Vulkan core artifact from the exact source/toolchain either matches its record
  or fails closed; it never becomes a Khronos-published selector or a compatibility claim.
