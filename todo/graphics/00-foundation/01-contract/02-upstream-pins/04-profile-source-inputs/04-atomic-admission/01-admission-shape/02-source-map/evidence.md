# F02.4.4.1.2 evidence

Revision: c9679935a3b4526582984085f4478db2dc771787
Validation: source map 8/8 + dependent audits 10/9/10/6/6/15 + source limit 6/6 + roadmap + diff + make test
Result: PASS
Artifacts: map sha256=7f3ee1a9e4faeb201de3775517f64d3f32f8b0805fd45b4efcfbea017b770785; contract sha256=8e859681d3e7ac641f2ab7be17ef2d0facc11b6fd95b377eca3941c6a7d1801a
Profile: planning-only six-source provenance map; no inventory, guest, API, browser, CTS, conformance, or performance behavior

Task ID and date: F02.4.4.1.2, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: c9679935a3b4526582984085f4478db2dc771787; clean tree after
the feature commit and before this receipt-only status update.
Upstream manifest revision: all audited roots bind inventory SHA-256
08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6; no inventory changed.
Guest image and build hashes: not applicable; this stdlib-only map does not build or run a guest.
Browser, OS, adapter and driver: not applicable; this task has no browser or GPU execution path.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python 3
with `PYTHONDONTWRITEBYTECODE=1` ran the three candidate audits, Vulkan include test, GLES closure
test, `source_map_test.py`, and `source_fetch_test.py`; then `cargo test -p emulator --test
source_file_limits --quiet`, `python3 scripts/check_graphics_roadmap.py`, `git diff --check`, and
`make test`.
Expected result and minimum nonzero case count: eight map tests pass; exactly six canonical F03 IDs
produce three `candidate-accepted` complete single sources, one unadmitted GLES closure, and two
unadmitted unresolved Vulkan roots.
Actual passed/failed/skipped counts and exit codes: OpenGL audit 10/0/0, GLES audit 9/0/0, Vulkan
audit 10/0/0, include 6/0/0, closure 6/0/0, map 8/0/0, fetch policy 15/0/0, source limit 6/0/0;
roadmap 227 documents, 140 tasks, 42 complete; all exit 0. `make test` exited 0 with Cargo 1,127
passed, 0 failed, 3 ignored and Node 337 passed, 0 failed.
Negative/reference checks and observed output: omission, duplicate, reorder, role swap, boolean schema,
stale/retyped roots, a fabricated Vulkan `members` key, lost blockers, stale observations, invalid
auxiliary paths, and invalid audit mappings raise `SourceMapError`; CLI reports 3/1/2 source shapes.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external payload,
image, or log was retained; reproduce eight temporary-fixture cases with `source_map_test.py`.
Artifact hashes: `source_map_inputs.py` sha256=27d3d5b4fcb9296c7de8388855498264cba1ce88c9a7e7a030468e854627a5de;
`source_map_test.py` sha256=7f311f689be8774e1e1779ede9bb4310e12ee0997e1a8fe4cc0c515d8d96bc47.
Software fallback detection and actual execution route: not applicable; no graphics route is executed.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: no focused test failed. Vulkan Docs generated,
transitive, and core-scope closure plus VCTS per-member, oversize, and core-versus-WSI/extension scope
remain unresolved blockers; their 73 includes and 98 references are observations, not members.
Decision and limits of the evidence: accept only a pre-admission shape map. GLES retains four exact core
identities, one exclusion, and 12 configurations / 12,477 cases / 30,574 runs; Vulkan retains no
`members` schema or admitted state. No cache, provenance, F03, guest API, browser, CTS execution,
conformance, or performance result changed.
Commit/push verification: feature commit c9679935a3b4526582984085f4478db2dc771787 is local; this
receipt/status commit and exact remote SHA verification follow. Remote CI was not run.
Next ready task: F02.4.4.1.3 — bound unresolved Vulkan closures.
