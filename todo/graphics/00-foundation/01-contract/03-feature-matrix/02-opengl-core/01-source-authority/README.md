# F03.2.1 — Set the OpenGL source-authority boundary

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.2.1
Depends: F03.1, F02.5.4.2
Evidence: [receipt](evidence.md)

## Outcome

Every proposed OpenGL 4.6 locator class has an explicit role-aware authority decision before an inventory
reader exists. The boundary uses the sealed F02 contract, cannot resolve the historical manifest by alias,
and fails closed when a required locator needs an unadmitted distinct source.

## Starting points

- [sealed F02 source contract](../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)
- [F03 role-aware bindings](../../01-profile-scope/role_aware_bindings.py)
- [OpenGL parent task](../README.md)

## Checklist

- [x] Load the exact OpenGL normative-root and full-suite-root records only through the F03 role-aware
  binding; reject a legacy manifest alias, ambient module, wrong profile, or auxiliary record.
- [x] Classify command/object/state, limit/format, shader, and extension locators as derivable from the
  admitted normative root or as requiring a separately recorded source-admission decision.
- [x] Record the exact revision, digest, source locator syntax, and permitted extraction scope for every
  accepted class; do not infer a registry or GLSL authority from an unrelated record.
- [x] Make an unresolved class visibly unavailable to F03.2.2 and F03.2.3 rather than replacing it with
  a lower API version, compatibility-profile behavior, or an unpinned document.
- [x] Add positive and hostile boundary checks for stale, mixed, reordered, legacy, and auxiliary input.
- [x] Attach a no-claim boundary receipt and retain `matrix-incomplete`.

## Verification

A source ambiguity stops downstream extraction. This task neither imports an API row nor makes a support,
guest, browser, CTS-execution, certification, or performance claim.
