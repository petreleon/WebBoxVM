# F03.2 — Import the OpenGL 4.6 core inventory

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F03.2
Depends: F03.1, F02.5.4.2
Evidence: pending

Prerequisite lists: [F03.1](../01-profile-scope/README.md) and [the active role-aware gate]
(../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md).

## Outcome

Every mandatory OpenGL 4.6 core command, state, feature, limit, format, and shader requirement has a
stable source locator, an implementation owner, and a reference-test obligation.

## Starting points

- [sealed role-aware source contract](../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)
- [active F03 role-aware gate](../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md)
- [F05.2 profile-bound registration](../../../02-reproducibility/02-check-runner/02-profile-bound-registration/README.md)
- [VirGL capability evidence](../../../../../../research/virgl-compatibility.md)
- [profile schema](../01-profile-scope/README.md)

## Checklist

- [x] [F03.2.1 — Set the OpenGL source-authority boundary](01-source-authority/README.md)
- [ ] [F03.2.2 — Extract command, object, and state rows](02-command-object-state-inventory/README.md)
- [ ] [F03.2.3 — Extract limit, format, shader, and extension rows](03-limit-format-shader-inventory/README.md)
- [ ] [F03.2.4 — Map implementation ownership and CTS obligations](04-ownership-and-cts-obligations/README.md)
- [ ] [F03.2.5 — Register checks and aggregate the receipt](05-registration-and-aggregate-receipt/README.md)

## Verification

- No mandatory OpenGL 4.6 core row lacks an admitted source locator, owner, or independent reference-test
  obligation. A changed source identity, missing row, duplicate row, or unsupported row marked supported
  fails a focused check.
- The profile remains `matrix-incomplete` until F03.2.5 records all sources, rows, owners, reference-test
  obligations, and F05.2 registrations. Completion does not claim a guest API, browser path, CTS run,
  certification, support, or performance result.

## Split rationale

F02.5.4.2 admits an OpenGL 4.6 normative root and full-suite root, but it does not silently promote the
historical registry or GLSL aliases to authoritative profile inputs. Source authority therefore has a
separate failure boundary from two independent inventory families, ownership/test planning, and F05
registration. The first child either records an accepted locator derivation from the admitted normative
root or identifies the precise source-policy decision needed before a distinct input can be used; it does
not change the sealed F02 contract by implication.
