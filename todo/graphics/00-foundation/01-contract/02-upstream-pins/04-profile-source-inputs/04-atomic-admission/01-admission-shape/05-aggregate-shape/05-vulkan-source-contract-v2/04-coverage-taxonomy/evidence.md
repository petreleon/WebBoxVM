# F02.4.4.1.5.5.4 evidence

Revision: 1ba52d1f30197a99516f61119812742e9db87de0
Validation: taxonomy/report 4/4 + checker unit 10/10 + `make test` (Rust 1,127 pass/3 ignored; Node 337 pass)
Result: PASS
Artifacts: taxonomy canonical sha256=752a3ae5ff10ea0d9f9a2638c0d1612fb7af3d376dabad1601f8b9c1e76cb2f7;
validator file sha256=23fcc496f7b849c26a75a29757eafd10c312b6f76cc1f2a19b1b5b69b42b4d79;
report file sha256=979b5a2816784f953174c2c8cae52648a17727b7f1772bed0c3103ce5a7c6e69;
test file sha256=2d771a82f75501c6a17ab1f00af75e64805adf7ddfa8093b7ccb0f4c3f869929
Profile: planning-only V2 Vulkan CTS coverage taxonomy; no guest, CTS, browser, or performance run

Task ID and date: F02.4.4.1.5.5.4, 2026-09-10 Europe/Bucharest.
Tested commit and dirty diff hash: `1ba52d1f30197a99516f61119812742e9db87de0`; task-local paths were
clean after the commit. Independent cache-contract work remained uncommitted and outside this task scope.
Upstream manifest revision: annotated `vulkan-cts-1.4.6.2` tag object
`42c723aa10d2652590f02741827aef43b0421d23`, peeled commit
`f6a29701220f34dd1407513bfe80d74ca7b392ce`; root `vk-default.txt` SHA-256
`b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4`.
Guest image and build hashes: not applicable; this task validates source/report metadata only.
Browser, OS, adapter and driver: not applicable; no browser or GPU route was exercised.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python
3.14.6 ran `coverage_taxonomy_test.py` and `scripts/test_check_graphics_roadmap.py`; `make test`, `git diff
--check`, and a physical-line count covered the full task and repository checks.
Expected result and minimum nonzero case count: four hermetic methods must preserve all 98 real direct
selector paths plus one synthetic recursive tail, classify only six explicit non-core selectors, emit
both diagnostic report modes, and reject missing, duplicate, reordered, hash-modified, stale,
reclassified, and filtered data.
Actual passed/failed/skipped counts and exit codes: taxonomy/report 4/0/0 and checker 10/0/0 exited 0.
`make test` exited 0: Rust 1,127/0/3, source limits 6/0/0, and Node 337/0/0. All six maintained task
files are at most 179 physical lines; scoped whitespace check exited 0.
Negative/reference checks and observed output: the fixture records 0 core, 1 WSI, 1 video, 4 extension,
and 93 unknown members. `unknown` is non-core, and a zero-test member makes a diagnostic incomplete,
not clean. Re-sealed stale, reordered, reclassified, and hash-modified reports still fail because the
expected root, ordered ledger, per-member hashes, and frozen taxonomy digest are rebuilt.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no remote payload or GPU
capture was retained. Reproduce with `python3 .../04-coverage-taxonomy/coverage_taxonomy_test.py`; the
taxonomy JSON, validator, report builder, and test SHA-256 values are above.
Software fallback detection and actual execution route: not applicable; no guest/browser execution ran.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: none. The full-worktree `make test` exited 0 and its
roadmap subcheck reported 295 documents, 178 tasks, 49 PASS-complete, 27 superseded, with `.3.1` next ready.
Decision and limits of the evidence: six V1-observed non-core selectors receive exact VCTS paths and
locators. No filename is inferred to be Vulkan 1.4 core; all other direct or recursive members remain
unknown. A clean complete-suite diagnostic is not a compatibility, conformance, or Khronos certification claim.
Commit/push verification: `1ba52d1f30197a99516f61119812742e9db87de0` was pushed; `git ls-remote`
returned that exact SHA for `refs/heads/codex/graphics-f01-baseline`.
Next ready task: F02.4.4.1.5.5.5 — hand off the V2 aggregate after the closure-cache receipt is integrated.
