# F03.2.2.3.3 — Aggregate coverage and raw handoff guard

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.2.3.3
Depends: F03.2.2.3.1, F03.2.2.3.2
Evidence: pending

## Outcome

A source-ordered aggregate of reviewed state and lifecycle inventories with an explicit coverage
decision and raw-only handoff for F03.2.2.4. Three buffer-binding facts cannot satisfy this outcome.

## Starting points

- [Buffer-binding slice](../01-buffer-binding-lifecycle-raw-slice/README.md)
- [Remaining state and lifecycle families](../02-other-state-lifecycle-families/README.md)
- [Raw handoff and import guard](../../04-raw-handoff-and-import-guard/README.md)

## Checklist

- [ ] Bind every child artifact and reviewed table/section range to the exact same normative PDF.
- [ ] Reconcile ordered coverage, omissions, overlaps and numeric-limit/format routing with F03.2.3.
- [ ] Fail on missing, duplicate, reordered, mixed or cross-profile child facts and stale manifests.
- [ ] Preserve separate table facts and explicit prose transitions with their precise source anchors.
- [ ] Reject raw facts used as Matrix rows or promoted to support, owners/tests or CTS evidence.
- [ ] Attach an aggregate no-claim receipt and complete the parent only when its full outcome is met.

## Verification

Keep `complete=false` while any admitted state or lifecycle family remains unreviewed. Coverage
completion records source inventory only; it never establishes a complete operational transition
grammar or implementation. Claims remain false and Matrix/CTS counters remain zero.
