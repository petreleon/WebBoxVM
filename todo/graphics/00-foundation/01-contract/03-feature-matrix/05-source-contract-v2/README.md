# F03.5 — Audit admitted source use across inventories

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.5
Depends: F03.1, F02.5.4.2, F03.2, F03.3, F03.4
Evidence: pending

Prerequisite lists: [F03.1](../01-profile-scope/README.md), [the active role-aware gate]
(../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md),
and the three imported inventories.

## Outcome

After the three inventories exist, one audit proves each actual matrix locator consumes the admitted
role-aware binding without changing the OpenGL 4.6 core, GLES 3.2, or Vulkan 1.4 core targets. Source
availability remains distinct from guest compatibility, conformance, performance, and certification.

## Starting points

- [active F02.5.4.2 gate](../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md)
- [F03 v2 profile contract](../01-profile-scope/profile_contract_v2.py)
- [v2 source requirements](../01-profile-scope/source_requirements_v2.json)
- [v2 matrix contract](../01-profile-scope/matrix_contract_v2.py)

## Checklist

- [ ] Audit every imported row and locator through the active role-binding API.
- [ ] Require the exact OpenGL, GLES, and Vulkan normative-root/full-suite-root pairs; reject aliases,
  auxiliary records, stale closure evidence, cross-profile use, and filtered suite substitutions.
- [ ] Keep Vulkan 1.4 core as the inventory target while reporting core, WSI, video, and extension
  categories only as coverage boundaries; classification must not alter the unfiltered CTS root.
- [ ] Preserve blocked rows and profiles absent independent implementation evidence; an audit cannot
  promote a guest API, browser device, renderer, or test result.
- [ ] Add focused audit checks, run required gates, and attach a receipt with identities, closure
  evidence, and the still-separate compatibility obligations.

## Verification

- Every imported row resolves to the immutable F02.5.4.1 identity already admitted by F02.5.4.2.
- A pass here proves only source-use consistency, not a guest-visible implementation, conformance,
  performance, or Khronos status.
- The Vulkan matrix keeps its core target explicit; no local selector rewrite can hide WSI, video, or
  extension coverage from the canonical full-suite record.
