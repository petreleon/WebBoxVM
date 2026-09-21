# F03.3.2 — Extract the GLES API inventory

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.3.2
Depends: F03.3.1
Evidence: pending

## Outcome

The admitted GLES 3.2 command/state and limit/format vocabulary is extracted as reviewable raw source
facts with stable locators. A later ownership-and-reference task may import those facts into the shared
matrix only after it can attach real implementation owners and independent full-suite obligations.

## Starting points

- [source-authority boundary](../01-source-authority/README.md)
- [F03 v2 matrix contract](../../01-profile-scope/matrix_contract_v2.py)
- [GLES parent task](../README.md)

## Checklist

- [x] [F03.3.2.1 — Verify the normative PDF cache](01-normative-pdf-cache/README.md)
- [ ] [F03.3.2.2 — Extract raw command, object, and state facts](02-command-object-state-raw-inventory/README.md)
- [x] [F03.3.2.3 — Extract raw limit and format facts](03-limit-format-raw-inventory/README.md)
- [x] [F03.3.2.4 — Record unavailable shader, precision, and extension decisions](04-unavailable-language-extension-ledger/README.md)
- [ ] [F03.3.2.5 — Handoff raw facts and guard matrix import](05-raw-handoff-and-import-guard/README.md)

## Verification

The emitted raw facts are bounded source inventory only. They leave ownership, independent test
obligations, CTS execution, guest behavior, browser behavior, certification, and performance unresolved.

## Split rationale

F03.3.1 admits only `command-state` and `limit-format` from the GLES 3.2 normative PDF. Shader,
precision, and extension semantics require the separately unadmitted ESSL source, so their decision ledger
is deliberately not an API extraction. `matrix_contract_v2.py` requires both a concrete implementation
owner and a full-suite test role; raw PDF facts cannot truthfully supply either. Cache identity, raw fact
families, unavailable-source decisions, and the import guard are therefore separate fail-closed inputs.
F03.3.3 must establish ownership and independent obligations before any later matrix import; this split
does not extend the sealed F02 contract or imply GLES support.
