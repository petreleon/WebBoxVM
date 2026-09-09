# F02.4.4.1.5.1 evidence

Revision: 185012284c720d2718b73d0097314003a7d6698b
Validation: aggregate 8/8 + transition/source-map/boundary 8/8/9/9 + source limit 6/6 + roadmap + diff + make test
Result: PASS
Artifacts: contract sha256=e51f2b402fca1ea54ff464f83fd673b15fe57fc2b0c0102f40724d8c9cc6e834;
model sha256=af14b940063a7f25486da726c802623792495f4786afd6aca5f55ba0ce5faaa6;
tests sha256=8afb0f9c115fa48eb1eaed00897d5a122a716f02726ddcbb2fcc9e99bb685f1e
Profile: source-transition aggregation only; no inventory admission, guest, API, browser, CTS,
conformance, or performance behavior

Task ID and date: F02.4.4.1.5.1, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: `185012284c720d2718b73d0097314003a7d6698b`; the source tree
was clean after the feature commit and before this receipt-only status update.
Upstream manifest revision: all six retained source records remain bound to inventory SHA-256
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`; this task did not mutate it.
Guest image and build hashes: not applicable; this stdlib-only source-state aggregate does not build or
run a guest. Browser, OS, adapter and driver: not applicable; no graphics execution route exists here.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python
3.14.6 with `PYTHONDONTWRITEBYTECODE=1` ran `aggregate_test.py`, `post_cutover_test.py`,
`source_map_test.py`, and `boundary_test.py`. Cargo 1.93.0 ran
`cargo test -p emulator --test source_file_limits --quiet`; the roadmap checker, `git diff --check`,
and `make test` ran with Node v26.0.0.
Expected result and minimum nonzero case count: eight aggregate tests pass; the only valid result has
six ordered required IDs, three direct candidates, three exact blockers, GLES 4/1/12 facts, Vulkan
73/98 observations, and zero cutover-ready inputs.
Actual passed/failed/skipped counts and exit codes: aggregate 8/0/0, transition 8/0/0, source map
8/0/0, boundaries 9/0/0, and source limit 6/0/0; every command exited 0. `make test` reported Cargo
1,127 passed, 0 failed, 3 ignored and Node 337 passed, 0 failed, 0 skipped. The roadmap checker reported
235 documents, 144 tasks, and 46 complete, with only F02.4.4.1.5.2 and F02.4.4.1.5.3 ready.
Negative/reference checks and observed output: hostile cases reject an omitted/reordered/role-swapped
source shape; root-only GLES or fabricated Vulkan members; Vulkan fallback or scope weakening;
stale/incomplete audit bundles; mixed or partial policy; and invalid-first/valid-last duplicate source
map JSON. The frozen result constructor rejects caller-supplied state. CLI output is exactly
`AGGREGATE: pre-admission, 6 required inputs, 3 blockers, 0 cutover-ready`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external payload,
image, or log was retained. Reproduce the hermetic fixture cases with `aggregate_test.py`; the three
artifact digests are recorded above.
Software fallback detection and actual execution route: not applicable; no graphics route is executed.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: no focused check failed. The retained blockers are
`requires-multifile-core-selector-closure`,
`vulkan-docs-core-generated-closure-unadmitted`, and
`vcts-vk-default-compound-oversize-core-scope-unadmitted`.
Decision and limits of the evidence: this is a correctly blocked pre-admission result, not a closure,
successor inventory, cache, provenance renewal, F03 transition, graphics API, guest interface,
browser path, CTS run, conformance result, or performance measurement. F02.4.4.1.5 remains open until
the Vulkan Docs and VCTS closure tasks plus final admission proof complete.
Commit/push verification: feature commit `185012284c720d2718b73d0097314003a7d6698b` was pushed and
verified at `origin/codex/graphics-f01-baseline` with zero ahead/behind divergence. Remote CI was not run.
Next ready task: F02.4.4.1.5.2 and F02.4.4.1.5.3 — resolve the independently blocked Vulkan closures.
