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

- [ ] Extract only core command declarations, object taxonomy, and normative state-table facts from the
  verified PDF under the `command-state` decision.
- [ ] Record explicit lifecycle or state-transition rules only with their triggering command and source
  section; never infer a transition from an implementation or registry.
- [ ] Record a stable raw identity, physical PDF page, numeric section, source order, and derivation class
  for every fact while preserving the F03.3.1 locator syntax.
- [ ] Exclude desktop OpenGL, lower GLES versions, registry entries, extensions, ESSL semantics, limits,
  and format properties.
- [ ] Bind ordered declaration and table coverage so missing, duplicate, reordered, cross-profile, or
  source-identity-mismatched facts fail.
- [ ] Add focused raw-inventory and hostile-input checks, then attach a no-claim receipt.

## Verification

This child creates no Matrix v2 rows. It leaves owner, reference-test plan, CTS execution, guest, browser,
certification, and performance state unresolved.
