# F03.3.2.3 — Extract raw limit and format facts

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.3.2.3
Depends: F03.3.2.1
Evidence: pending

## Outcome

A separate bounded raw inventory records admitted GLES 3.2 core limits and format properties with stable
PDF locators. It keeps numeric and format facts distinct from command/state facts and matrix ownership.

## Starting points

- [verified normative-PDF cache](../01-normative-pdf-cache/README.md)
- [source-authority boundary](../../01-source-authority/README.md)
- [command/object/state raw inventory](../02-command-object-state-raw-inventory/README.md)

## Checklist

- [ ] Extract only core limit and format-property requirements covered by the `limit-format` decision.
- [ ] Preserve exact units, conditions, physical PDF page, numeric section, source order, and derivation
  class for every raw fact.
- [ ] Keep format support requirements distinct from an observed renderer format capability or a claim of
  implementation support.
- [ ] Exclude command/state facts, desktop or lower-version behavior, registry entries, extensions, and
  unadmitted ESSL shader or precision semantics.
- [ ] Bind ordered section coverage and serialized-size limits; reject missing, duplicate, reordered,
  stale, cross-profile, or source-identity-mismatched facts.
- [ ] Add focused raw-inventory and hostile-input checks, then attach a no-claim receipt.

## Verification

The raw set is not a Matrix v2 document. Owner, independent test obligation, CTS execution, guest,
browser, certification, and performance evidence remain unresolved.
