# F03.2.3.1 — Extract raw limit and format facts

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.2.3.1
Depends: F03.2.1, F03.2.2.1
Evidence: pending

## Outcome

An ordered, bounded raw inventory records only OpenGL 4.6 core limit and format facts admitted by
F03.2.1. It is source data for later ownership work, not a Matrix v2 record or an implementation result.

## Starting points

- [source-authority boundary](../../01-source-authority/README.md)
- [normative-PDF cache boundary](../../02-command-object-state-inventory/01-normative-pdf-cache/README.md)
- [future matrix contract](../../../01-profile-scope/matrix_contract_v2.py)

## Checklist

- [ ] Consume only the F03.2.1 `limit-format` class through the verified external PDF-cache boundary.
- [ ] Record each explicit core limit or format property with raw identity, physical page, numeric section,
  table or row anchor, source order, and derivation class.
- [ ] Bind a reviewed table-and-section coverage manifest and serialized-size cap so sampling cannot look
  complete and a missing family cannot disappear.
- [ ] Exclude shader semantics, extensions, registry-only names, compatibility behavior, and implementation
  observations from the raw inventory.
- [ ] Reject stale or mixed source bytes, altered anchors, duplicate or reordered facts, cross-profile
  facts, and false supported status; attach a no-claim receipt.
- [ ] Produce only a blocked raw handoff for F03.2.3.3 with no owner or reference-test fields.

## Verification

The source locator keeps the F03.2.1 PDF grammar; table and row detail belongs in the raw condition. This
child does not create Matrix v2 rows, execute CTS, or establish guest, browser, certification, or
performance behavior.
