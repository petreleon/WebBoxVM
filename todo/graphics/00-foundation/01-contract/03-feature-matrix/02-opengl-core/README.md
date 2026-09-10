# F03.2 — Import the OpenGL 4.6 core inventory

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.2
Depends: F02, F03.5
Evidence: pending

Prerequisite lists: [F02](../../02-upstream-pins/README.md) and
[F03.5](../05-source-contract-v2/README.md).

## Outcome

Every mandatory OpenGL 4.6 core command, state, feature, limit, format, and shader requirement has a
stable source locator, an implementation owner, and a reference-test obligation.

## Starting points

- [OpenGL/GLES registry input](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [GLSL 4.60 input](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [VirGL capability evidence](../../../../../../research/virgl-compatibility.md)
- [profile schema](../01-profile-scope/README.md)

## Checklist

- [ ] Extract the OpenGL 4.6 core required commands, state, limits, formats, and shader rules from the
  pinned inputs into the shared row schema with exact source locators.
- [ ] Record core-profile scope and every extension decision explicitly; keep compatibility-profile
  behavior and unadopted extensions outside the final target until a recorded decision adds them.
- [ ] Link every mandatory row to a downstream implementation task and an independent native/guest or
  conformance reference test; leave absent mappings visibly unassigned or blocked.
- [ ] Classify current bounded VirGL behavior only where present evidence supports it; do not advertise
  an untested row as supported or silently replace it with a lower bring-up version.
- [ ] Keep this leaf blocked if F03.5 has not admitted the authoritative API/limit/format source and
  complete conformance-suite closure; do not infer either from the registry or one isolated test case.
- [ ] Add focused extraction/coverage/negative tests, run required gates, and attach a receipt.

## Verification

- No mandatory OpenGL 4.6 core row lacks a source locator, owner, or reference-test plan.
- A changed registry identity, missing row, duplicate row, or unsupported row marked supported fails
  the focused check.
