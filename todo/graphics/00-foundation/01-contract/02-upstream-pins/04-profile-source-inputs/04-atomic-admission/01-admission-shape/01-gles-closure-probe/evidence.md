# F02.4.4.1.1 evidence

Revision: 9cac6a151d4b1837c1d02ae27c30f15138095883
Validation: closure 6/6 + GLES audit 9/9 + fetch policy 15/15 + source limit 6/6 + roadmap + diff + make test
Result: PASS
Artifacts: `closure_shape_contract.py` sha256=46582abbe3b08e0f803aa9762817c0817290259e1d3a2f9579ab88c525bc3d97; `closure_shape_test.py` sha256=8bd96741db7e10a17e9253d831ec695d01ebb4700dc1b1cf17bba9caa0e55fde
Profile: GLES 3.2 source-provenance probe only; no inventory, guest, API, browser, conformance, or performance behavior

Task ID and date: F02.4.4.1.1, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: 9cac6a151d4b1837c1d02ae27c30f15138095883; clean tree after
the feature commit and before this receipt-only status update.
Upstream manifest revision: F02 candidate audits bind inventory SHA-256
08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6; no inventory was changed.
Guest image and build hashes: not applicable; this stdlib-only probe does not build or run a guest.
Browser, OS, adapter and driver: not applicable; this task has no browser or GPU execution path.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python 3
with `PYTHONDONTWRITEBYTECODE=1` ran `closure_shape_test.py`, the existing GLES candidate audit,
and `source_fetch_test.py`; then `cargo test -p emulator --test source_file_limits --quiet`,
`python3 scripts/check_graphics_roadmap.py`, `git diff --check`, and `make test`.
Expected result and minimum nonzero case count: six new hostile/positive tests pass; an exact GLES
root plus four core selectors and 12 configurations returns only `unadmitted`.
Actual passed/failed/skipped counts and exit codes: closure 6/0/0; GLES audit 9/0/0; fetch policy
15/0/0; source limit 6/0/0; roadmap 227 documents, 140 tasks, 42 complete; all exit 0. `make test`
exited 0 with Cargo 1,127 passed, 0 failed, 3 ignored and Node 337 passed, 0 failed.
Negative/reference checks and observed output: root-only, missing or stale member, mutable URL,
oversize byte count, duplicate id/cache/selector, configuration drift, and extension reinsertion raise
`ShapeError`; the positive candidate audit remains `("accepted", "rejected")`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external payload,
image, or log was retained; reproduce the six temporary-fixture cases with `closure_shape_test.py`.
Software fallback detection and actual execution route: not applicable; no graphics route is executed.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: Vulkan Docs generated/transitive closure and VCTS
member, oversize, WSI/video/extension scope remain unresolved outside this child; no probe test failed.
Decision and limits of the evidence: accept only an audit-only GLES logical closure. The root remains
rejected, the returned state has no admitted constructor, and no F02 inventory, cache, F03, or consumer
changed. This does not establish CTS execution, guest support, browser behavior, or performance.
Commit/push verification: feature commit 9cac6a151d4b1837c1d02ae27c30f15138095883 is local; this
receipt/status commit and remote SHA verification follow before handoff.
Next ready task: F02.4.4.1.2 — map all required-source shapes.
