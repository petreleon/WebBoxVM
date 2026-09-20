# F03.3.3 — Map shader and CTS obligations

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.3.3
Depends: F03.3.1, F03.3.2
Evidence: pending

## Outcome

Every admitted GLES 3.2 API row has explicit shader or precision scope where applicable, a downstream
implementation owner, and an independent native, guest, or canonical conformance-test obligation. Missing
mappings remain visible instead of shrinking the profile.

## Starting points

- [source-authority boundary](../01-source-authority/README.md)
- [GLES API inventory](../02-api-inventory/README.md)
- [admitted GLES CTS root](../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)
- [VirGL capability evidence](../../../../../../../research/virgl-compatibility.md)

## Checklist

- [ ] Consume shader and precision locator classes only when F03.3.1 accepts them; otherwise record the
  exact blocker rather than assuming desktop GLSL or a lower ESSL version.
- [ ] Merge API and shader coverage without losing a mandatory GLES 3.2 row; reject a missing, duplicate,
  or source-incompatible row before assigning ownership.
- [ ] Link every mandatory row to a concrete downstream implementation task or a visibly unresolved owner;
  a generic transport milestone is not an owner.
- [ ] Give each row an independent native, guest, or canonical conformance-test obligation with an exact
  selector or a documented reason it remains blocked.
- [ ] Classify bounded VirGL evidence only where it supports the exact row; reject empty owners, generic
  test plans, stale suite roots, wrong profiles, and rows marked supported without independent evidence.
- [ ] Attach a coverage receipt with zero implied CTS executions and no qualification claims.

## Verification

The coverage map plans shader scope, implementation, and testing. It does not execute CTS or establish
GLES guest behavior, browser behavior, certification, support, or performance.
