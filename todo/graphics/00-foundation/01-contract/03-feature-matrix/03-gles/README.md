# F03.3 — Import the GLES 3.2 inventory

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.3
Depends: F02, F03.1
Evidence: pending

Prerequisite lists: [F02](../../02-upstream-pins/README.md) and
[F03.1](../01-profile-scope/README.md).

## Outcome

Every mandatory GLES 3.2 command, state, feature, limit, format, and shader requirement has a stable
source locator, an implementation owner, and a reference-test obligation.

## Starting points

- [OpenGL/GLES registry input](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [ESSL 3.20 input](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [VirGL capability evidence](../../../../../../research/virgl-compatibility.md)
- [profile schema](../01-profile-scope/README.md)

## Checklist

- [ ] Extract GLES 3.2 required commands, state, limits, formats, and shader rules from the pinned
  inputs into the shared row schema with exact source locators.
- [ ] Record the GLES core and extension boundary explicitly, including version-specific shader and
  precision requirements rather than assuming desktop OpenGL equivalence.
- [ ] Link every mandatory row to a downstream implementation task and independent native/guest or
  conformance reference test; retain missing mappings as unassigned or blocked.
- [ ] Classify current bounded behavior only from evidence, preserving earlier bring-up versions as
  milestones rather than changing the GLES 3.2 target.
- [ ] Keep this leaf blocked if F03.1 finds the authoritative API/limit/format source or complete test
  manifest missing; do not infer it from the registry or one isolated test case.
- [ ] Add focused extraction/coverage/negative tests, run required gates, and attach a receipt.

## Verification

- No mandatory GLES 3.2 row lacks a source locator, owner, or reference-test plan.
- A stale ESSL/registry identity, missing row, duplicate row, or unproven supported row fails the
  focused check.
