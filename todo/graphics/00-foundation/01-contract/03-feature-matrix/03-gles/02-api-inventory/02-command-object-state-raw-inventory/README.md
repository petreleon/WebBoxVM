# F03.3.2.2 — Extract raw command, object, and state facts

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.3.2.2
Depends: F03.3.2.1
Evidence: pending

## Outcome

A bounded, self-hashed raw inventory records the admitted GLES 3.2 core command, object, and state
vocabulary in source order. It is source data, not an implementation-owner or Matrix v2 record.

## Starting points

- [verified normative-PDF cache](../01-normative-pdf-cache/README.md)
- [source-authority boundary](../../01-source-authority/README.md)
- [future matrix schema](../../../01-profile-scope/matrix_contract_v2.py)

## Checklist

- [x] [F03.3.2.2.1 — Classify the command/state domain](01-command-domain-classification/README.md)
- [ ] [F03.3.2.2.2 — Normalize literal and templated declarations](02-template-declaration-grammar/README.md)
- [ ] [F03.3.2.2.3 — Extract object and resource command slices](03-object-resource-command-slices/README.md)
- [ ] [F03.3.2.2.4 — Extract state and execution command slices](04-state-execution-command-slices/README.md)
- [ ] [F03.3.2.2.5 — Aggregate command/state coverage and handoff](05-aggregate-coverage-handoff/README.md)

## Verification

Only F03.3.2.2.5 may close the classified command/state domain. Every slice remains raw-only and creates
no Matrix v2 rows; owner, reference-test plan, CTS execution, guest/browser behavior, certification, and
performance remain unresolved.
