# F02.5 — Admit source roles without conflating authority

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F02.5
Depends: F02.1, F02.2, F02.3.1, F03.1
Evidence: pending

## Outcome

Each target profile receives immutable Khronos source roots and a reviewable local derivation record.
The WebBoxVM record never claims that Khronos authored a local transform. Full unmodified CTS or
must-pass suites stay distinct from engineering case maps and remain mandatory at final acceptance.

## Starting points

- [historical F02.4 plan](../04-profile-source-inputs/README.md)
- [reviewed candidates](../04-profile-source-inputs/candidate_catalog.py)
- [F03 source requirements](../../03-feature-matrix/01-profile-scope/source_requirements.json)
- [F02 fetch contract](../02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [x] [F02.5.1 — Define source authority and transform roles](01-authority-and-transform-boundary/README.md)
- [ ] [F02.5.2 — Record normative source roots and local builds](02-normative-build-record/README.md)
- [ ] [F02.5.3 — Record unmodified full conformance-suite roots](03-full-suite-record/README.md)
- [ ] [F02.5.4 — Cut over inventory and source consumers atomically](04-inventory-and-consumer-cutover/README.md)

## Verification

- Every profile binds a pinned normative source and an independently pinned full conformance suite.
- A local build, shard, or case map records WebBoxVM as producer and cannot claim Khronos authority,
  full-suite coverage, conformance, certification, or profile support.
- The final qualification lane runs the exact admitted full suite; a local engineering subset may only
  accelerate development and must leave unmapped mandatory requirements visibly blocked.
