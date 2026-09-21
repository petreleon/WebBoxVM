# F03.2.2 — Extract command, object, and state rows

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.2.2
Depends: F03.2.1
Evidence: pending

## Outcome

The OpenGL 4.6 core command, object, state, and lifecycle vocabulary is extracted as a complete,
reviewable raw source inventory with stable locators. A later ownership task imports those raw facts into
the shared matrix only after it can attach real implementation owners and independent test obligations.

## Starting points

- [source-authority boundary](../01-source-authority/README.md)
- [F03 v2 matrix contract](../../01-profile-scope/matrix_contract_v2.py)
- [OpenGL parent task](../README.md)

## Checklist

- [x] [F03.2.2.1 — Verify the normative PDF cache](01-normative-pdf-cache/README.md)
- [ ] [F03.2.2.2 — Extract raw command and object facts](02-command-object-raw-inventory/README.md)
- [ ] [F03.2.2.3 — Extract raw state and lifecycle facts](03-state-and-lifecycle-raw-inventory/README.md)
- [ ] [F03.2.2.5 — Cover remaining command and object vocabulary](05-remaining-command-object-vocabulary/README.md)
- [ ] [F03.2.2.4 — Handoff raw facts and guard matrix import](04-raw-handoff-and-import-guard/README.md)

## Verification

The emitted raw facts are bounded source inventory only. They leave ownership, independent test
obligations, CTS execution, guest behavior, browser behavior, certification, and performance unresolved.

## Split rationale

`matrix_contract_v2.py` requires an implementation owner and a full-suite test role, neither of which a
PDF extraction can truthfully invent. Cache identity, command/object facts, and state/lifecycle facts are
therefore independently fail-closed raw inputs. Direct-creation facts are not the complete command
vocabulary: F03.2.2.5 keeps the remaining vocabulary visibly open. F03.2.4 owns the later matrix import
after it maps real owners and reference obligations; this split does not weaken F03.2.1 or add a source to
F02.
