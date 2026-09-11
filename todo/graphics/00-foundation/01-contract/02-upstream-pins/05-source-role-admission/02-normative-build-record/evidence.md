# F02.5.2 aggregate evidence

Revision: 91d18e32
Validation: three child receipts, fresh F02.5.2.3 admission, make test, source-file limits, roadmap check
Result: PASS
Artifacts: four pinned source records, bounded local output SHA-256 29b9b64f67eca6c8ddf3edd40bddcb0843c059403dd757ec3479c2939dfd7556
Profile: normative source roots and one local engineering definition; no full-suite qualification

## Child closure

- F02.5.2.1: immutable roots and payload notice checks, receipt at
  [01-normative-root-pins/evidence.md](01-normative-root-pins/evidence.md).
- F02.5.2.2: bounded WebBoxVM VK_VERSION_1_4 XML facts, receipt at
  [02-vulkan-local-definition/evidence.md](02-vulkan-local-definition/evidence.md).
- F02.5.2.3: fresh four-source refresh, staged builder, rebuild, artifact verification, and no-claim
  receipt at [03-admission-receipt/evidence.md](03-admission-receipt/evidence.md).

The builder and exact CPython runtime are pinned; the local artifact is 7,506 bytes,
well below 8 MiB. Its VK_VERSION_1_4 XML facts do not reproduce Vulkan prose, create a
Khronos selector, or claim guest support, compatibility, CTS execution, conformance,
certification, profile support, or performance.

## Aggregate decision

F02.5.2 is complete only as source admission for a local engineering artifact. The
independently required full CTS and must-pass suite roots remain F02.5.3 work; nothing
in this receipt narrows, replaces, or executes them.

First failing subcheck: no source or artifact mismatch; the receipt records the
initial sandbox DNS limitation and successful approved fresh refresh.
Commit/push verification: child implementation 91d18e32 committed locally; no remote push performed.
Next ready task: F02.5.3.1 canonical full-suite roots.
