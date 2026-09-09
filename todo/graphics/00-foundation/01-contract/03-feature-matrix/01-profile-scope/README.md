# F03.1 — Define the profile scope and matrix schema

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.1
Depends: F02.1
Evidence: pending

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

- [ ] Record the final target names: OpenGL 4.6 core, GLES 3.2, and Vulkan 1.4 core; explicitly
  distinguish compatibility profiles, extensions, and earlier bring-up versions.
- [ ] Define a small row schema for source locator, mandatory status, implementation owner, reference
  test, current evidence, and supported/emulated/unsupported/blocked classification.
- [ ] Bind the schema and every future matrix artifact to the reviewed F02 inventory-lock identity and
  source IDs, rejecting a stale or invented source reference.
- [ ] Audit whether F02 pins the authoritative API/limit/format material and complete test manifests
  required for all three final profiles; create a visible F02 extension blocker when it does not.
- [ ] Add focused positive and malformed/stale-schema checks with an exact nonzero test count.
- [ ] Run focused tests, source limits, roadmap, whitespace, required gates, and attach a receipt.

## Verification

- The schema cannot mark an API profile complete without a version, scope, source locator, owner, and
  independent test reference for every mandatory row.
- Current VirGL/WebGPU and blob work is classified as bounded evidence or blocked, never as an
  untested OpenGL, GLES, Venus, or Vulkan feature.
- A missing authoritative source or case catalog produces a concrete F02 blocker; it cannot become a
  fabricated inventory row or a lower final profile.

## Maintained contract

`profile_scope.json` fixes only the three final target scopes. They remain `blocked`: while any of
the six exact sources is absent the blocker is `inventory-sources-incomplete`; after an F02 lock
renewal admits all six, it must change to `matrix-incomplete`, not to profile support.

`source_requirements.json` names those six future inventory IDs and the reviewed existing inputs
they extend. `validate_profile_scope.py` rejects a stale lock, a substituted ID, a missing role, or
an invented related source; its current live result is intentionally `BLOCKED`.

Future F03.2–F03.4 artifacts use `matrix_contract.py` through
`validate_profile_scope.py --matrix PATH`. Every row binds a normative source and a distinct,
profile-specific CTS/must-pass source by ID, revision, and digest, plus a concrete locator, owner,
test selector, status, blocker, and local `evidence.md` receipt. The contract rejects cross-profile
sources, placeholders, stale identities, unsupported schema fields, and a supported/emulated row
without concrete evidence. It validates a row's shape and provenance, not coverage or runtime support.
