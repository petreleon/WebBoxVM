# F03.2.2.2 — Extract raw command and object facts

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.2.2.2
Depends: F03.2.2.1
Evidence: pending

## Outcome

A bounded, self-hashed raw inventory records the admitted OpenGL 4.6 core command and object vocabulary
in source order. It is source data, not an implementation-owner or Matrix v2 record.

## Starting points

- [verified normative-PDF cache](../01-normative-pdf-cache/README.md)
- [source-authority boundary](../../01-source-authority/README.md)
- [future matrix schema](../../../01-profile-scope/matrix_contract_v2.py)

## Checklist

- [ ] Extract only formal command declarations and object taxonomy from the verified normative PDF.
- [ ] Apply the documented C-binding `gl` prefix rule only to command names that the PDF declares without
  that display prefix.
- [ ] Record a stable raw identity, physical PDF page, numeric section, source order, and derivation class
  for every command or object fact.
- [ ] Exclude compatibility-only behavior, unadmitted registry entries, extensions, and shading-language
  semantics.
- [ ] Bind an ordered section-coverage manifest and serialized-size cap so a missing family cannot pass.
- [ ] Reject altered pages, template-expansion mistakes, duplicate/reordered facts, and false supported
  status; attach a no-claim receipt.

## Verification

This child does not create Matrix v2 rows. It leaves owner, reference-test plan, CTS execution, guest,
browser, certification, and performance state unresolved.
