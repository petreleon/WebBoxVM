# F02.4.4.1.3 evidence

Revision: f7c5402453a9a9566428f3b176e9c4ee3b70a62e
Validation: boundaries 9/9 + Vulkan audit 10/10 + includes 6/6 + source map 8/8 + fetch policy 15/15 + source limit 6/6 + roadmap + diff + make test
Result: PASS
Artifacts: boundaries sha256=a676b29421b2681e074264175dbae758dfbdb56e7edfd48b5858ba17513a1a90; contract sha256=26663f148d3a87dbcd86cc77f77dbb2e9d86524b20e4f6792f2247fe1c0f69d8
Profile: Vulkan 1.4 core pre-admission boundary only; no inventory, guest, API, browser, CTS, conformance, or performance behavior

Task ID and date: F02.4.4.1.3, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: f7c5402453a9a9566428f3b176e9c4ee3b70a62e; clean tree after
the feature commit and before this receipt-only status update.
Upstream manifest revision: audited root records bind inventory SHA-256
08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6; no inventory changed.
Guest image and build hashes: not applicable; this stdlib-only boundary record does not build or run a guest.
Browser, OS, adapter and driver: not applicable; this task has no browser or GPU execution path.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python 3
with `PYTHONDONTWRITEBYTECODE=1` ran the Vulkan candidate, include, source-map, boundary, and fetch
tests; then `cargo test -p emulator --test source_file_limits --quiet`, `python3
scripts/check_graphics_roadmap.py`, `git diff --check`, and `make test`.
Expected result and minimum nonzero case count: nine boundary tests pass; exactly two canonical Vulkan
roots remain `unresolved-root` and `unadmitted`, with 73 and 98 transcript observations and zero
admitted closures.
Actual passed/failed/skipped counts and exit codes: Vulkan audit 10/0/0, includes 6/0/0, source map
8/0/0, boundaries 9/0/0, fetch policy 15/0/0, source limit 6/0/0; roadmap 228 documents, 140 tasks,
43 complete; all exit 0. `make test` exited 0 with Cargo 1,127 passed, 0 failed, 3 ignored and Node
337 passed, 0 failed.
Negative/reference checks and observed output: fabricated member, cache, and inventory fields; false
state/shape/fallback; missing or swapped blockers; Docs scope/configuration erosion; VCTS scope
expansion and oversize weakening; stale source map; and duplicate JSON fields raise `BoundaryError`.
CLI reports `BOUNDARIES: 2 unresolved Vulkan roots, 0 admitted closures`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external payload,
image, or log was retained; reproduce nine temporary-fixture cases with `boundary_test.py`.
Artifact hashes: `boundary_test.py` sha256=da6d6f5d69bc6aa4b7e24aafc68b59194ca2672e246f911d9199d59e17907ae9.
Software fallback detection and actual execution route: not applicable; no graphics route is executed.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: no focused test failed. Vulkan Docs generated,
transitive, macro/conditional, and core-scope configuration identities remain unresolved. Per the
[F02.4.3 audit evidence](../../../03-vulkan-input-audit/evidence.md), the inspected committed VCTS
member set at `f6a29701220f34dd1407513bfe80d74ca7b392ce`, bound to root
`b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4`, has 14 members over F02.2's
8 MiB (8,388,608-byte) limit and 434,669,348 aggregate bytes, but identities remain unpinned;
WSI, video, and extension-group scope remains outside any core admission.
Decision and limits of the evidence: preserve both roots as pre-admission boundaries only. The VCTS
size figures are audit observations, not a member inventory or closure. No cache, provenance, F03,
guest API, browser, CTS execution, conformance, or performance result changed.
Commit/push verification: feature commit f7c5402453a9a9566428f3b176e9c4ee3b70a62e is local; this
receipt/status commit and exact remote SHA verification follow. Remote CI was not run.
Next ready task: F02.4.4.1.4 — define post-cutover rules.
