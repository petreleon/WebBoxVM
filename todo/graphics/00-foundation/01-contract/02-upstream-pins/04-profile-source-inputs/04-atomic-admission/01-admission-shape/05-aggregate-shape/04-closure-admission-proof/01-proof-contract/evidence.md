# F02.4.4.1.5.4.1 evidence

Revision: 7b8022017bd21866dde2d94ca035e457bcb779cd
Validation: proof contract 5/5 + roadmap tests 10/10 + source limit 6/6 + roadmap + diff + make test
Result: PASS
Artifacts: contract JSON sha256=b9b5b0b4d06f326cc156b3ecd4fc840a77b3ba44656a89977fb6a3f6c620afa2;
validator sha256=0b59709c91fdce59dd373f48d7d1aad551fd61076cec9aa1bcbe7d076ebb8083;
tests sha256=b28a02d1766b28a629fc94967b5bd908167cf4973a175bb12d0e9f27d8696fa3
Profile: proof-grammar only; six frozen source roles, all readiness states false; no API implementation

Task ID and date: F02.4.4.1.5.4.1, 2026-09-10 Europe/Bucharest.
Tested commit and dirty diff hash: `7b8022017bd21866dde2d94ca035e457bcb779cd`; the worktree was clean
after the feature commit and before this receipt-only status update.
Upstream manifest revision: the frozen source-requirements file is bound by raw SHA-256
`a9535611bab654034357d1578c3a2035caa09fda8090114df0867ac6d7fbeca5`; its six F03 rows retain inventory
lock `08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.
Guest image and build hashes: not applicable; this stdlib-only grammar neither builds nor runs a guest.
Browser, OS, adapter and driver: not applicable; no graphics execution route exists here.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python
3.14.6 with `PYTHONDONTWRITEBYTECODE=1` ran `admission_proof_contract.py` and its five-test suite;
`scripts/test_check_graphics_roadmap.py`, `scripts/check_graphics_roadmap.py`, `git diff --check`, and
`make test` ran with Cargo 1.93.0 and Node v26.0.0.
Expected result and minimum nonzero case count: five focused cases pass; the only valid grammar has six
ordered F03 tuples, three V1 regular sources, one V1 compound closure, a V1 unresolved Docs boundary,
and a V1 VCTS boundary with supplementary V2 diagnostic evidence. All five readiness flags are false.
Actual passed/failed/skipped counts and exit codes: proof contract 5/0/0, roadmap tests 10/0/0, and
source limit 6/0/0; every command exited 0. `make test` reported Cargo 1,127 passed, 0 failed, 3 ignored,
and Node 337 passed, 0 failed, 0 skipped.
Negative/reference checks and observed output: hostile cases reject reordered, eroded, or schema-changed
F03 requirements; missing/duplicate/reordered roles; Docs↔VCTS envelope substitution; true admission or
F03 readiness; stale locks; extra fields; duplicate JSON keys; and oversized JSON. The CLI prints
`CONTRACT: 6 roles a9535611bab654034357d1578c3a2035caa09fda8090114df0867ac6d7fbeca5 non-admitting`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no payload, image, or raw log
is retained. Re-run the checked Python test beside the three hashed artifacts listed above.
Software fallback detection and actual execution route: not applicable; no renderer or fallback runs.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: no focused test failed. This contract deliberately
keeps `vulkan-14-spec` unresolved and forbids V2 VCTS from satisfying Docs or core-selector semantics.
Decision and limits of the evidence: PASS means the proof grammar is self-hashed, bounded, and fail-closed.
It does not admit any source, resolve Docs, prove a fresh VCTS cache, create an inventory/cutover/F03
artifact, or claim guest API, browser, compatibility, CTS conformance, certification, or performance.
Commit/push verification: feature commit `7b8022017bd21866dde2d94ca035e457bcb779cd` was pushed to
`origin/codex/graphics-f01-baseline`; the receipt commit is verified separately after its push. Remote CI
was not run.
Next ready task: F02.4.4.1.5.4.2 — reconcile current evidence fail-closed.
