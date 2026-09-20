# F03.2.1 evidence

Revision: b7fc8c96ff63fb0b383a69b66b7267ee0177ae86
Validation: `make graphics-opengl-source-authority-test`; `make test`; `git diff --check`; `python3 scripts/check_graphics_roadmap.py`
Result: PASS
Artifacts: `opengl_source_authority.json` SHA-256 `19318f772dc18618cc53173bc67f92ee3c137a90b10a5ee24d7c8b2a3572b419`
Profile: OpenGL 4.6 core source-boundary only; `blocked` / `matrix-incomplete`

Task ID and date: F03.2.1, 2026-09-21.

Tested commit and dirty diff hash: tested from the revision above; the focused source diff before this
receipt had SHA-256 `11c9d43486912acd38df497502d8621195b2ef1537ba888d53d2bcb28633cbbc`.

Upstream manifest revision: deliberately not consumed. The only authority input is the fixed-path F03
role-aware binding, which raw-lock-loads F02.5.4.1 source contract
`d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3`.

Exact commands, working directory and tool versions: from repository root with Python 3,
`make graphics-opengl-source-authority-test`, then `make test`, `git diff --check`, and
`python3 scripts/check_graphics_roadmap.py`.

Expected result and minimum nonzero case count: six focused boundary tests pass; malformed or ambient
source inputs fail closed; the complete local suite exits zero.

Actual passed/failed/skipped counts and exit codes: focused target ran 6 passed, 0 failed (exit 0).
`make test`, whitespace check, and roadmap checker exited 0; the checker reported 413 documents,
246 tasks, 62 PASS-complete, and 81 superseded before this task was checked.

Negative/reference checks and observed output: stale self-hash, stale/mixed/reordered roots, legacy
registry alias, Vulkan auxiliary record, wrong profile, stale raw F02 lock, ambient module, invalid
locator, and shader/extension consumption each fail. The positive CLI reports two derivable and two
unavailable classes while remaining `matrix-incomplete`.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external capture was
created; rerun the commands above. The self-hashed boundary artifact is the retained review record.

Software fallback detection and actual execution route: not applicable; this is a local provenance
validator, not guest, browser, renderer, or CTS execution.

First failing subcheck or blocker: no local test failed. Shader and extension extraction remain blocked
by `unadmitted-distinct-source`; the historical GLSL/OpenGL-registry inputs are not aliases here.

Decision and limits of the evidence: command/object/state and limit/format locators may use
`opengl46-core-pdf-v1:page=<positive-decimal>;section=<section-path>` from the sealed normative root.
This proves neither API support nor CTS execution, conformance, certification, browser behavior, or
performance; `cts_executions` remains zero and every qualification claim is false.

Commit/push verification: not committed or pushed by request.

Next ready task: F03.2.2 may consume only the two derivable classes. A separate F02 admission decision
is required before F03.2.3 can consume shader or extension locators.
