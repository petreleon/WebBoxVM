# F03.3 — Import the GLES 3.2 inventory

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.3
Depends: F03.1, F02.5.4.2
Evidence: pending

Prerequisite lists: [F03.1](../01-profile-scope/README.md) and [the active role-aware gate]
(../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md).

## Outcome

Every mandatory GLES 3.2 command, state, feature, limit, format, and shader requirement has a stable
source locator, an implementation owner, and a reference-test obligation.

## Starting points

- [sealed role-aware source contract](../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)
- [active F03 role-aware gate](../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md)
- [OpenGL/GLES registry candidate](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [ESSL 3.20 candidate](../../02-upstream-pins/01-input-inventory/manifest.toml)
- [VirGL capability evidence](../../../../../../research/virgl-compatibility.md)
- [profile schema](../01-profile-scope/README.md)
- [F05.2 profile-bound registration](../../../02-reproducibility/02-check-runner/02-profile-bound-registration/README.md)

## Checklist

- [ ] [F03.3.1 — Set the GLES source-authority boundary](01-source-authority/README.md)
- [ ] [F03.3.2 — Extract the GLES API inventory](02-api-inventory/README.md)
- [ ] [F03.3.3 — Map shader and CTS obligations](03-shader-and-cts-obligations/README.md)
- [ ] [F03.3.4 — Register checks and aggregate the receipt](04-registration-and-aggregate-receipt/README.md)

## Verification

- No mandatory GLES 3.2 row lacks a source locator, owner, or reference-test plan.
- A stale ESSL/registry identity, missing row, duplicate row, or unproven supported row fails the
  focused check.

## Split rationale

F02.5.4.2 admits GLES 3.2 normative and full-suite roots, but it does not silently promote a historical
registry or ESSL candidate into an authoritative locator. Source authority is therefore isolated from the
API inventory, shader and CTS coverage obligations, and F05 registration. The first child either records
an accepted derivation from the admitted normative root or leaves the affected input visibly blocked; it
does not extend the sealed F02 contract by implication.
