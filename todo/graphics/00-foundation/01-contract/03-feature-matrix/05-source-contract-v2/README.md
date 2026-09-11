# F03.5 — Adopt source-contract v2

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.5
Depends: F03.1, F02.5.4
Evidence: pending

Prerequisite lists: [F03.1](../01-profile-scope/README.md) and
[F02.5.4](../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/README.md).

## Outcome

F03 consumes the renewed, lock-bound source contract for every matrix input without changing the
OpenGL 4.6 core, GLES 3.2, or Vulkan 1.4 core targets. Source availability is distinct from guest
compatibility, conformance, performance, and Khronos certification.

## Starting points

- [F02 role-aware source cutover](../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/README.md)
- [F03 profile contract](../01-profile-scope/profile_contract.py)
- [source requirements](../01-profile-scope/source_requirements.json)
- [matrix contract](../01-profile-scope/matrix_contract.py)

## Checklist

- [ ] Consume F02.5.4's atomic role-aware source identities and receipts for every required profile
  role; reject a stale, partial, mixed, root-only, or substituted source contract.
- [ ] Bind `vulkan-cts-mustpass` to the canonical unfiltered upstream suite root and its admitted
  record, not a locally filtered "core-only" selector, isolated CTS file, or unverified aggregate.
- [ ] Keep Vulkan 1.4 core as the inventory target while recording core, WSI, video, and extension
  suite categories only as auditable coverage/reporting boundaries; classification must not alter CTS.
- [ ] Preserve `matrix-incomplete` as a source-sufficiency state only; it must not promote a matrix
  row, guest API, browser device, renderer, or test result to supported, conformant, or certified.
- [ ] Add focused positive and hostile adoption checks, run required gates, and attach a receipt with
  exact source identities, closure evidence, and the still-separate compatibility obligations.

## Verification

- F03 refuses a source contract whose canonical root, recursive closure, lock identity, or declared
  scope differs from the F02 admission receipt.
- A pass here proves only that matrix work may use verified source material. It is not proof of a
  guest-visible implementation, Vulkan/OpenGL/GLES conformance, performance, or Khronos status.
- The Vulkan matrix keeps its core target explicit; no local selector rewrite can hide WSI, video, or
  extension coverage from the canonical CTS-suite record.
