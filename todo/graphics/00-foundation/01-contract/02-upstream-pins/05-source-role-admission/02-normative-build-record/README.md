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

- [ ] [F02.5.2.1 — Pin and re-fetch normative roots](01-normative-root-pins/README.md)
- [ ] [F02.5.2.2 — Build a bounded local Vulkan 1.4 definition](02-vulkan-local-definition/README.md)
- [ ] [F02.5.2.3 — Admit records and publish the no-claim receipt](03-admission-receipt/README.md)

## Verification

- Rebuilding the declared Vulkan core artifact from the exact source/toolchain either matches its record
  or fails closed; it never becomes a Khronos-published selector or a compatibility claim.

## Split rationale

The source pins, a WebBoxVM-produced Vulkan facts artifact, and the final admission/receipt have
different authorities and failure modes. The split prevents a successful local extraction from being
mistaken for proof that the normative roots were refreshed or that Khronos published a core selector.
