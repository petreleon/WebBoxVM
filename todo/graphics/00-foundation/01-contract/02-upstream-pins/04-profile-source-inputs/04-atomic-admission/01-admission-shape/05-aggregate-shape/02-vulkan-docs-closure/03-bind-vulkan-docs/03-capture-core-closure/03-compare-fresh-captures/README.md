# F02.4.4.1.5.2.3.3.3 — Compare fresh captures and record the receipt

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.3.3
Depends: F02.4.4.1.5.2.3.3.1, F02.4.4.1.5.2.3.3.2
Evidence: pending

Prerequisite lists: the [input observer](../01-observe-pinned-build-inputs/README.md),
[input/scope model](../02-bind-core-input-scope/README.md), and
[official build witness](../../01-reproduce-pinned-html/evidence.md).

## Outcome

Two independently located pinned-build captures have the same normalized input and scope identities and a compact,
reproducible unadmitted receipt records their nonzero evidence. The comparison binds the prior output witness without
staging or re-hashing its payload tree.

## Starting points

- [build witness](../../01-reproduce-pinned-html/evidence.md)
- [actual Docs grammar](../../02-actual-closure-identity/evidence.md)
- [capture observer](../01-observe-pinned-build-inputs/README.md)

## Checklist

- [ ] Capture two fresh, independently located source/output observations and validate each through the input/scope
  grammar.
- [ ] Require equal normalized input, condition, producer, and scope identities while rejecting reused locations,
  stale/copy records, or any difference.
- [ ] Bind the known two-run output-tree identity as a witness relation only; leave output-tree staging and re-hashing to
  child `.4`.
- [ ] Record a tracked receipt with nonzero raw/derived/condition/promotion counts and ignored full artifacts, then run
  focused tests, roadmap, source-limit, and project regression checks.

## Verification

- Equal rendered output does not make a missing or mismatched input closure acceptable.
- This result remains unadmitted and changes no F02/V1 inventory, candidate, F03, guest, browser, CTS, conformance, or
  performance state.
