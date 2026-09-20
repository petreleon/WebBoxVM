# F03.3.4 — Register checks and aggregate the receipt

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.3.4
Depends: F03.3.2, F03.3.3, F05.2
Evidence: pending

## Outcome

The complete GLES 3.2 API inventory, shader and CTS obligations, and profile-bound F05 registrations are
joined in one reviewable receipt. Missing source, device, browser, guest, or suite prerequisites remain
blocked.

## Starting points

- [GLES API inventory](../02-api-inventory/README.md)
- [shader and CTS obligations](../03-shader-and-cts-obligations/README.md)
- [F05.2 profile-bound registration](../../../../02-reproducibility/02-check-runner/02-profile-bound-registration/README.md)

## Checklist

- [ ] Register every admitted row family through F05.2 with its exact GLES profile, source identity,
  expected nonzero count, command, artifact location, and mandatory prerequisites.
- [ ] Reject a stale source, wrong profile, empty command, missing row family, missing owner, missing
  independent test obligation, or unavailable prerequisite presented as PASS.
- [ ] Run the focused source-boundary, API-inventory, shader/CTS-coverage, and F05 registration checks;
  preserve actual blocked results and commands in the receipt.
- [ ] Aggregate source identities, row counts, extension decisions, shader scope, owners, test obligations,
  and F05 registrations into one self-consistent review record.
- [ ] Mark the F03.3 parent complete only after reviewing every child receipt and every blocked mandatory
  row; otherwise retain `matrix-incomplete`.
- [ ] Attach the aggregate no-claim receipt with exact commands, outputs, and zero implied CTS executions.

## Verification

Registration proves only that planned checks are bound to frozen inputs. It cannot turn an unavailable
browser, native device, guest image, shader source, or conformance suite into a pass, nor claim GLES
support, certification, or near-native performance.
