# F02.4.4.1.4 evidence

Revision: 270be30d500c1ec62f1f1acdfd549e7ff11fcebb
Validation: transition 8/8 + dependent source checks 8/9/10/9/10/6/15 + source limit 6/6 + roadmap + diff + make test
Result: PASS
Artifacts: rules sha256=74300113dab0019c25b1ed629d13c7fbba720316f370a7deee1cdb86b56d91bc; contract sha256=460f848bb5e351368c1adf49a70fc9c4ed8349c4f9014f63364d3c72a5373fdd
Profile: source-transition design only; no inventory admission, guest, API, browser, CTS, conformance, or performance behavior

Task ID and date: F02.4.4.1.4, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: 270be30d500c1ec62f1f1acdfd549e7ff11fcebb; clean tree after
the feature commit and before this receipt-only status update.
Upstream manifest revision: all audited roots remain bound to inventory SHA-256
08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6; no inventory changed.
Guest image and build hashes: not applicable; this stdlib-only transition design does not build or run a guest.
Browser, OS, adapter and driver: not applicable; this task has no browser or GPU execution path.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python
3.14.6 with `PYTHONDONTWRITEBYTECODE=1` ran the source-map, boundary, transition, three candidate-audit,
Vulkan-include, and fetch-contract suites. Then Cargo 1.93.0 ran `cargo test -p emulator --test
source_file_limits --quiet`; `python3 scripts/check_graphics_roadmap.py`, `git diff --check`, and
`make test` ran with Node v26.0.0.
Expected result and minimum nonzero case count: eight transition tests pass; the only result is
`PreAdmission` with all six canonical IDs, three exact blockers, and zero cutover-ready inputs.
Actual passed/failed/skipped counts and exit codes: source map 8/0/0, boundaries 9/0/0, transition
8/0/0, OpenGL audit 10/0/0, GLES audit 9/0/0, Vulkan audit 10/0/0, includes 6/0/0, fetch 15/0/0,
and source limit 6/0/0; roadmap reported 229 documents, 140 tasks, 44 complete; every command exited
0. `make test` reported Cargo 1,127 passed, 0 failed, 3 ignored and Node 337 passed, 0 failed, 0 skipped.
Negative/reference checks and observed output: the transition suite rejects mixed/partial/stale policy,
weakened future grammar, omitted/reordered/role-swapped source-map records, root-only GLES substitution,
Vulkan fallback, stale or incomplete audit bundles, and caller-supplied state. Invalid-first/valid-last
duplicate JSON in rules, source map, boundaries, and audit paths is rejected before a permissive parser
could select the valid final key. CLI output is exactly `TRANSITION: pre-admission, 6 required inputs,
0 cutover-ready`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external payload, image,
or log was retained. Reproduce the eight temporary-fixture cases with `post_cutover_test.py`; contract,
schema, and test SHA-256 values are 460f848bb5e351368c1adf49a70fc9c4ed8349c4f9014f63364d3c72a5373fdd,
fcc1c7996b7cb3c5540637d46eee4c1f54e892b6b3a60456365ba11fed361835, and
d0d57c992de1e7b7955ba497994085d4d29ec6a52fc62f76267e18e4f6ecbc3f.
Software fallback detection and actual execution route: not applicable; no graphics route is executed.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: no focused check failed. The retained blockers are
`requires-multifile-core-selector-closure`, `vulkan-docs-core-generated-closure-unadmitted`, and
`vcts-vk-default-compound-oversize-core-scope-unadmitted`.
Decision and limits of the evidence: the rules freeze a future closure/receipt grammar, including full
F02.2 member identities, typed scope joins, predecessor admission-shape digest, unchanged legacy audit
history through a successor adapter, and four same-cutover consumer bindings. They do not create a
closure, receipt, successor inventory, cache, provenance renewal, F03 transition, graphics API, guest
interface, browser path, CTS run, conformance result, or performance measurement. F02.4.4.1.5 must
validate actual future artifact instances before any cutover-ready result is possible.
Commit/push verification: feature commit 270be30d500c1ec62f1f1acdfd549e7ff11fcebb is local; this
receipt/status commit and exact remote SHA verification follow. Remote CI was not run.
Next ready task: F02.4.4.1.5 — aggregate the admission shape.
