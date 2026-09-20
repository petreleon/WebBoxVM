# F02.5 aggregate evidence

Revision: `f5337681`
Validation: F02.5.1–F02.5.4 child receipts; full local suite; source-file limits, diff, and roadmap checks
Result: PASS
Artifacts: [role-authority receipt](01-authority-and-transform-boundary/evidence.md), [normative-build
receipt](02-normative-build-record/evidence.md), [full-suite receipt](03-full-suite-record/evidence.md),
and [cutover receipt](04-inventory-and-consumer-cutover/evidence.md)
Profile: immutable source provenance and closure only; no runtime API or qualification result

Task ID and date: F02.5, 2026-09-20 Europe/Bucharest. Every target profile now binds one pinned
normative root and one independently pinned unfiltered full-suite root through the sealed eight-record
contract. WebBoxVM-derived artifacts remain auxiliary and cannot discharge a mandatory role.

The completed cutover binds the same contract to F03's role-aware source gate and a future-F05 accessor.
The active F03 state advances only to `matrix-incomplete`; F05 has no profile registration. The external
fresh selector cache contains nine small selector/license files; complete-suite closure evidence remains
separate and no CTS execution occurred.

The local mandatory gates passed: `make test`, source-file limits 6/6, whitespace diff, and roadmap
checker. There is no guest image, browser, renderer, driver, support, conformance, certification,
performance, or near-native result. Code revision `f5337681` is pushed to
`origin/codex/graphics-f01-baseline`; no Actions run is claimed. Next ready task: F03.2, F03.3, or F03.4.1.
