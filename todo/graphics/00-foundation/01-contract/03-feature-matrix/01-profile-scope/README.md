# F03.1 — Define the profile scope and matrix schema

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.1
Depends: F02.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.1](../../02-upstream-pins/01-input-inventory/README.md).

## Outcome

One reviewed, lock-bound schema states what a future OpenGL, GLES, or Vulkan completion must prove,
without treating the current bounded renderer as support for any profile.

## Starting points

- [replacement goal](../../../../goal.md)
- [F02 inventory](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [VirGL limits](../../../../../../research/virgl-compatibility.md)
- [Venus limits](../../../../../../research/venus-foundations.md)

## Checklist

- [x] Record the final target names: OpenGL 4.6 core, GLES 3.2, and Vulkan 1.4 core; explicitly
  distinguish compatibility profiles, extensions, and earlier bring-up versions.
- [x] Define a small row schema for source locator, mandatory status, implementation owner, reference
  test, current evidence, and supported/emulated/unsupported/blocked classification.
- [x] Bind the schema and every future matrix artifact to the reviewed F02 inventory-lock identity and
  source IDs, rejecting a stale or invented source reference.
- [x] Audit whether F02 pins the authoritative API/limit/format material and complete test manifests
  required for all three final profiles; create a visible F02 extension blocker when it does not.
- [x] Add focused positive and malformed/stale-schema checks with an exact nonzero test count.
- [x] Run focused tests, source limits, roadmap, whitespace, required gates, and attach a receipt.

## Verification

- The schema cannot mark an API profile complete without a version, scope, source locator, owner, and
  independent test reference for every mandatory row.
- Current VirGL/WebGPU and blob work is classified as bounded evidence or blocked, never as an
  untested OpenGL, GLES, Venus, or Vulkan feature.
- A missing authoritative source or case catalog produces a concrete F02 blocker; it cannot become a
  fabricated inventory row or a lower final profile.

## Maintained contract

`profile_scope.json`, `source_requirements.json`, `profile_contract.py`, and `matrix_contract.py`
remain the immutable v1 snapshot used by superseded F02.4 evidence. Their old inventory result is not
the active source gate.

The active successor is `profile_scope_v2.json`, `source_requirements_v2.json`, and
`validate_profile_scope.py`. It loads F02.5.4.1 only through its raw-byte lock, binds the three exact
normative-root/full-suite-root pairs, and returns success only while all profiles remain
`blocked` with `matrix-incomplete`. It does not accept a manifest switch or a legacy alias.

Future F03.2–F03.4 artifacts use `matrix_contract_v2.py` through the active runner. A row names only
the two roles; the validator resolves the pinned identity, unfiltered selector, and closure through the
same sealed binding API. It rejects auxiliary substitutes, stale headers, cross-profile roles,
unsupported schema fields, and any supported/emulated state. This validates provenance, not coverage
or runtime support.
